// Export inferred C pseudocode and per-function recovery status from Ghidra.
// @category OSFR
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.FunctionIterator;
import java.io.BufferedWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

public class ExportNative extends GhidraScript {
    private String cell(String text) {
        return text == null ? "" : text.replace('\t', ' ').replace('\r', ' ').replace('\n', ' ');
    }

    @Override
    public void run() throws Exception {
        long started = System.nanoTime();
        String[] args = getScriptArgs();
        if (args.length != 1) throw new IllegalArgumentException("Expected a new output directory");
        Path output = Path.of(args[0]);
        Files.createDirectory(output);
        DecompInterface decompiler = new DecompInterface();
        decompiler.toggleCCode(true);
        decompiler.toggleSyntaxTree(false);
        if (!decompiler.openProgram(currentProgram)) throw new IllegalStateException("Cannot open program in decompiler");
        int recovered = 0, failed = 0, external = 0;
        int discovered = currentProgram.getFunctionManager().getFunctionCount();
        try (BufferedWriter code = Files.newBufferedWriter(output.resolve("pseudocode.c"), StandardCharsets.UTF_8);
             BufferedWriter index = Files.newBufferedWriter(output.resolve("functions.tsv"), StandardCharsets.UTF_8)) {
            code.write("/* Ghidra-inferred pseudocode; not original source and not a compilable Rust port. */\n");
            index.write("address\tname\tstatus\tmessage\n");
            FunctionIterator functions = currentProgram.getFunctionManager().getFunctions(true);
            while (functions.hasNext() && !monitor.isCancelled()) {
                Function function = functions.next();
                if (function.isExternal()) { external++; continue; }
                DecompileResults result = decompiler.decompileFunction(function, 30, monitor);
                String address = function.getEntryPoint().toString();
                String name = cell(function.getName());
                if (result.decompileCompleted() && result.getDecompiledFunction() != null) {
                    code.write("\n/* " + address + " " + name + " */\n");
                    code.write(result.getDecompiledFunction().getC());
                    index.write(address + "\t" + name + "\trecovered\t\n");
                    recovered++;
                } else {
                    index.write(address + "\t" + name + "\tfailed\t" + cell(result.getErrorMessage()) + "\n");
                    failed++;
                }
                if ((recovered + failed) % 1000 == 0) {
                    println("OSFR_EXPORT recovered=" + recovered + " failed=" + failed);
                    code.flush(); index.flush();
                }
            }
        } finally { decompiler.dispose(); }
        String summary = "{\"program\":\"" + cell(currentProgram.getName()) + "\",\"discovered_functions\":" + discovered + ",\"external_functions_skipped\":" + external + ",\"recovered_functions\":" + recovered + ",\"failed_functions\":" + failed + ",\"cancelled\":" + monitor.isCancelled() + ",\"elapsed_seconds\":" + ((System.nanoTime() - started) / 1000000000L) + "}";
        Files.writeString(output.resolve("summary.json"), summary, StandardCharsets.UTF_8);
        println("OSFR_EXPORT_COMPLETE " + summary);
    }
}
