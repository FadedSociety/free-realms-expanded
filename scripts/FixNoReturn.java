// Repair functions wrongly marked non-returning by auto-analysis (e.g. MSVC _free and a pooled free routine):
// a function is wrongly no-return if its body (or its thunk target's body) contains a RET instruction.
// For each such function: clear the no-return flag, restore the fall-through of every call to it,
// disassemble the code after the call, and re-derive the calling function's body. Runs in memory only
// (use -readOnly; nothing is saved). Exports every function whose body changed, in ExportNativeMap's format.
// Args: <new output dir>. Writes functions.jsonl, fixed-noreturn.tsv, changed.tsv, summary.json.
// @category OSFR
import ghidra.app.script.GhidraScript;
import ghidra.app.cmd.function.CreateFunctionCmd;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.program.model.address.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import java.io.BufferedWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;

public class FixNoReturn extends GhidraScript {
    private static final int MAX_LIST = 200, MAX_STR = 300;

    private static String q(String s) {
        if (s == null) return "null";
        StringBuilder b = new StringBuilder("\"");
        for (char c : s.toCharArray()) {
            switch (c) {
                case '"': b.append("\\\""); break;
                case '\\': b.append("\\\\"); break;
                case '\n': b.append("\\n"); break;
                case '\r': b.append("\\r"); break;
                case '\t': b.append("\\t"); break;
                default:
                    if (c < 0x20) b.append(String.format("\\u%04x", (int) c)); else b.append(c);
            }
        }
        return b.append('"').toString();
    }

    private static String arr(Collection<String> items) {
        StringJoiner j = new StringJoiner(",", "[", "]");
        int n = 0;
        for (String s : items) { if (n++ >= MAX_LIST) break; j.add(q(s)); }
        return j.toString();
    }

    private static String fnRef(Function f) { return f.getEntryPoint().toString() + " " + f.getName(true); }

    private boolean bodyHasRet(Function f) {
        Function t = f.isThunk() ? f.getThunkedFunction(true) : f;
        if (t == null) return false;
        for (Instruction ins : currentProgram.getListing().getInstructions(t.getBody(), true)) {
            if (ins.getFlowType().isTerminal() && ins.getMnemonicString().toUpperCase().startsWith("RET")) return true;
        }
        return false;
    }

    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 1) throw new IllegalArgumentException("Expected a new output directory");
        Path output = Path.of(args[0]);
        Files.createDirectory(output);
        FunctionManager fm = currentProgram.getFunctionManager();
        Listing listing = currentProgram.getListing();
        ReferenceManager refs = currentProgram.getReferenceManager();

        // 1. wrongly no-return functions
        List<Function> wrong = new ArrayList<>();
        for (Function f : fm.getFunctions(true)) {
            if (f.hasNoReturn() && bodyHasRet(f)) wrong.add(f);
        }
        StringBuilder fixed = new StringBuilder();
        Map<Address, Long> before = new HashMap<>();
        for (Function f : fm.getFunctions(true)) before.put(f.getEntryPoint(), f.getBody().getNumAddresses());
        Set<Function> touched = new LinkedHashSet<>();
        for (Function f : wrong) {
            f.setNoReturn(false);
            int sites = 0;
            for (Reference r : refs.getReferencesTo(f.getEntryPoint())) {
                if (!r.getReferenceType().isCall()) continue;
                Instruction ins = listing.getInstructionAt(r.getFromAddress());
                if (ins == null) continue;
                ins.setFlowOverride(FlowOverride.NONE);
                ins.clearFallThroughOverride();
                Address next = ins.getMaxAddress().next();
                if (next != null && listing.getInstructionAt(next) == null) disassemble(next);
                Function host = fm.getFunctionContaining(ins.getAddress());
                if (host != null) touched.add(host);
                sites++;
            }
            fixed.append(fnRef(f)).append("\t").append(sites).append("\n");
            println("OSFR_NORET fixed " + fnRef(f) + " call_sites=" + sites);
        }
        Files.writeString(output.resolve("fixed-noreturn.tsv"), fixed.toString(), StandardCharsets.UTF_8);

        // 2. re-derive bodies of touched functions; collect those that changed
        List<Function> changed = new ArrayList<>();
        StringBuilder ch = new StringBuilder();
        for (Function f : touched) {
            CreateFunctionCmd.fixupFunctionBody(currentProgram, f, monitor);
            long b0 = before.getOrDefault(f.getEntryPoint(), -1L), b1 = f.getBody().getNumAddresses();
            if (b1 != b0) { changed.add(f); ch.append(f.getEntryPoint()).append("\t").append(b0).append("\t").append(b1).append("\n"); }
        }
        Files.writeString(output.resolve("changed.tsv"), ch.toString(), StandardCharsets.UTF_8);
        println("OSFR_NORET wrong=" + wrong.size() + " touched=" + touched.size() + " changed=" + changed.size());

        // 3. export the changed functions
        DecompInterface decompiler = new DecompInterface();
        decompiler.toggleCCode(true);
        decompiler.toggleSyntaxTree(false);
        if (!decompiler.openProgram(currentProgram)) throw new IllegalStateException("Cannot open program in decompiler");
        SymbolTable symbols = currentProgram.getSymbolTable();
        long started = System.nanoTime();
        int done = 0, failed = 0;
        try (BufferedWriter out = Files.newBufferedWriter(output.resolve("functions.jsonl"), StandardCharsets.UTF_8)) {
            for (Function f : changed) {
                if (monitor.isCancelled()) break;
                TreeSet<String> callees = new TreeSet<>(), callers = new TreeSet<>();
                for (Function c : f.getCalledFunctions(monitor)) callees.add(fnRef(c));
                for (Function c : f.getCallingFunctions(monitor)) callers.add(fnRef(c));
                LinkedHashSet<String> strings = new LinkedHashSet<>(), data = new LinkedHashSet<>(), vtables = new LinkedHashSet<>();
                for (Instruction ins : listing.getInstructions(f.getBody(), true)) {
                    for (Reference r : refs.getReferencesFrom(ins.getAddress())) {
                        Address to = r.getToAddress();
                        if (!to.isMemoryAddress() || r.getReferenceType().isFlow()) continue;
                        Data d = listing.getDataContaining(to);
                        Symbol s = symbols.getPrimarySymbol(to);
                        String sname = s == null ? null : s.getName(true);
                        if (d != null && d.hasStringValue()) {
                            String v = String.valueOf(d.getValue());
                            if (v.length() > MAX_STR) v = v.substring(0, MAX_STR) + "...";
                            strings.add(to + "=" + v);
                        } else if (sname != null && sname.contains("vftable")) {
                            vtables.add(to + " " + sname);
                        } else if (sname != null) {
                            data.add(to + " " + sname);
                        }
                    }
                }
                Function thunked = f.isThunk() ? f.getThunkedFunction(false) : null;
                DecompileResults res = decompiler.decompileFunction(f, 60, monitor);
                boolean ok = res.decompileCompleted() && res.getDecompiledFunction() != null;
                if (!ok) failed++;
                StringBuilder b = new StringBuilder(4096);
                b.append("{\"addr\":").append(q(f.getEntryPoint().toString()))
                 .append(",\"name\":").append(q(f.getName()))
                 .append(",\"namespace\":").append(q(f.getParentNamespace().getName(true)))
                 .append(",\"signature\":").append(q(f.getPrototypeString(false, false)))
                 .append(",\"convention\":").append(q(f.getCallingConventionName()))
                 .append(",\"source\":").append(q(f.getSymbol().getSource().toString()))
                 .append(",\"bytes\":").append(f.getBody().getNumAddresses())
                 .append(",\"thunk_of\":").append(thunked == null ? "null" : q(fnRef(thunked)))
                 .append(",\"callees\":").append(arr(callees))
                 .append(",\"callers\":").append(arr(callers))
                 .append(",\"n_callers\":").append(callers.size())
                 .append(",\"strings\":").append(arr(strings))
                 .append(",\"vtables\":").append(arr(vtables))
                 .append(",\"data\":").append(arr(data))
                 .append(",\"decompiled\":").append(ok)
                 .append(",\"error\":").append(ok ? "null" : q(res.getErrorMessage()))
                 .append(",\"c\":").append(ok ? q(res.getDecompiledFunction().getC()) : "null")
                 .append("}\n");
                out.write(b.toString());
                if (++done % 2000 == 0) { println("OSFR_NORET exported=" + done + " failed=" + failed); out.flush(); }
            }
        } finally { decompiler.dispose(); }
        String summary = "{\"wrong_noreturn\":" + wrong.size() + ",\"touched\":" + touched.size() + ",\"changed\":" + changed.size()
            + ",\"exported\":" + done + ",\"decompile_failed\":" + failed
            + ",\"elapsed_seconds\":" + ((System.nanoTime() - started) / 1000000000L) + "}";
        Files.writeString(output.resolve("summary.json"), summary, StandardCharsets.UTF_8);
        println("OSFR_NORET_COMPLETE " + summary);
    }
}
