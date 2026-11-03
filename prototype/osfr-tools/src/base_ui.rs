//! L952: BaseUi (opcode 47) packets the server sends to the client's UI.
//!
//! ExecuteScript (47/7) runs one client Lua method. The client reader (`FUN_00a91bf0` case 7)
//! splits the script at '.', calls `Table:Method` through its script call (`FUN_009797a0`) and
//! passes the ints as the method's arguments; a script without the dot is dropped silently
//! (native 2026-11-03). This is how the server starts the client's own tutorials, for example
//! `SoccerTutorial.StartSoccerTut` or `RacingTutorial.StartDemoDerbBasics`.
use std::io;

const OPCODE: u16 = 47;
const EXECUTE_SCRIPT: u8 = 7;
const MAX_SCRIPT: usize = 64;
const MAX_PARAMS: usize = 8;

/// Whether `script` is a `Table.Method` the client reader runs: two identifiers and one dot.
pub fn valid_script(script: &str) -> bool {
    let ident = |part: &str| {
        !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    };
    script.len() <= MAX_SCRIPT
        && script
            .split_once('.')
            .is_some_and(|(table, method)| ident(table) && ident(method))
}

/// The 47/7 ExecuteScript body: the script, then the int arguments.
pub fn execute_script(script: &str, params: &[i32]) -> io::Result<Vec<u8>> {
    if !valid_script(script) || params.len() > MAX_PARAMS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("client script {script:?} is not Table.Method with at most {MAX_PARAMS} ints"),
        ));
    }
    let mut out = Vec::with_capacity(3 + 4 + script.len() + 4 + 4 * params.len());
    out.extend(OPCODE.to_le_bytes());
    out.push(EXECUTE_SCRIPT);
    out.extend((script.len() as u32).to_le_bytes());
    out.extend(script.as_bytes());
    out.extend((params.len() as u32).to_le_bytes());
    for param in params {
        out.extend(param.to_le_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_soccer_tutorial_script_is_the_bytes_the_client_ran() {
        let bytes = execute_script("SoccerTutorial.StartSoccerTut", &[]).unwrap();
        let mut expected = vec![47, 0, 7, 29, 0, 0, 0];
        expected.extend(b"SoccerTutorial.StartSoccerTut");
        expected.extend([0, 0, 0, 0]);
        assert_eq!(bytes, expected);
        let with = execute_script("RacingTutorial.StartDemoDerbBasics", &[2]).unwrap();
        assert_eq!(&with[with.len() - 8..], &[1, 0, 0, 0, 2, 0, 0, 0]);
    }

    #[test]
    fn scripts_the_client_would_drop_are_refused() {
        for bad in [
            "SoccerTutorial:StartSoccerTut",
            "Start",
            ".Start",
            "Table.",
            "A.b.c",
            "A.b,c",
            "A b.c",
        ] {
            assert!(execute_script(bad, &[]).is_err(), "{bad}");
        }
        assert!(execute_script("A.b", &[0; MAX_PARAMS + 1]).is_err());
    }
}
