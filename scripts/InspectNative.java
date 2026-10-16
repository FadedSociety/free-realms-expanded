// Record native function coverage without modifying the saved Ghidra program.
// @category OSFR
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.FunctionIterator;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

public class InspectNative extends GhidraScript {
    @Override
    public void run() throws Exception {
        if (getScriptArgs().length != 1) throw new IllegalArgumentException("Expected a new JSON output path");
        int external = 0, internal = 0;
        FunctionIterator iterator = currentProgram.getFunctionManager().getExternalFunctions();
        while (iterator.hasNext()) { iterator.next(); external++; }
        iterator = currentProgram.getFunctionManager().getFunctions(true);
        while (iterator.hasNext()) { iterator.next(); internal++; }
        String result = "{\"total_functions\":" + currentProgram.getFunctionManager().getFunctionCount()
            + ",\"internal_functions\":" + internal + ",\"external_functions\":" + external + "}";
        Files.writeString(Path.of(getScriptArgs()[0]), result, StandardCharsets.UTF_8, StandardOpenOption.CREATE_NEW);
        println("OSFR_FUNCTION_INVENTORY " + result);
    }
}
