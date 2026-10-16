// Define functions at given entry addresses that Ghidra's auto-analysis missed (e.g. vtable slot
// targets), in memory only (run with -readOnly so nothing is saved), and export the newly created
// functions in the same JSON format as ExportNativeMap. Args: <targets file: one hex VA per line> <new output dir>.
// These are Ghidra inferences, not original source; mappers must cite them as such.
// @category OSFR
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import java.io.BufferedWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;

public class DefineAndExportNew extends GhidraScript {
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

    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 2) throw new IllegalArgumentException("Expected <targets file> <new output directory>");
        Path output = Path.of(args[1]);
        Files.createDirectory(output);
        FunctionManager fm = currentProgram.getFunctionManager();
        List<Function> created = new ArrayList<>();
        StringBuilder report = new StringBuilder();
        for (String line : Files.readAllLines(Path.of(args[0]), StandardCharsets.UTF_8)) {
            line = line.trim();
            if (line.isEmpty()) continue;
            Address a = toAddr(Long.parseLong(line, 16));
            if (fm.getFunctionAt(a) != null) { report.append(line).append("\texists\n"); continue; }
            Function host = fm.getFunctionContaining(a);
            if (host != null) { report.append(line).append("\tinside\t").append(fnRef(host)).append("\n"); continue; }
            if (getInstructionAt(a) == null) disassemble(a);
            Function f = createFunction(a, null);
            if (f == null) { report.append(line).append("\tcreate_failed\n"); continue; }
            created.add(f);
            report.append(line).append("\tcreated\t").append(f.getBody().getNumAddresses()).append("\n");
        }
        Files.writeString(output.resolve("define-report.tsv"), report.toString(), StandardCharsets.UTF_8);
        DecompInterface decompiler = new DecompInterface();
        decompiler.toggleCCode(true);
        decompiler.toggleSyntaxTree(false);
        if (!decompiler.openProgram(currentProgram)) throw new IllegalStateException("Cannot open program in decompiler");
        Listing listing = currentProgram.getListing();
        ReferenceManager refs = currentProgram.getReferenceManager();
        SymbolTable symbols = currentProgram.getSymbolTable();
        long started = System.nanoTime();
        int done = 0, failed = 0;
        try (BufferedWriter out = Files.newBufferedWriter(output.resolve("functions.jsonl"), StandardCharsets.UTF_8)) {
            Iterator<Function> it = created.iterator();
            while (it.hasNext() && !monitor.isCancelled()) {
                Function f = it.next();
                if (f.isExternal()) continue;
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
                if (++done % 2000 == 0) { println("OSFR_MAP exported=" + done + " failed=" + failed); out.flush(); }
            }
        } finally { decompiler.dispose(); }
        String summary = "{\"program\":" + q(currentProgram.getName()) + ",\"exported\":" + done + ",\"decompile_failed\":" + failed
            + ",\"cancelled\":" + monitor.isCancelled() + ",\"elapsed_seconds\":" + ((System.nanoTime() - started) / 1000000000L) + "}";
        Files.writeString(output.resolve("summary.json"), summary, StandardCharsets.UTF_8);
        println("OSFR_MAP_COMPLETE " + summary);
    }
}
