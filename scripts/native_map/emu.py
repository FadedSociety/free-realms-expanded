"""Run single functions of FreeRealms.exe under the Unicorn x86 emulator, for behaviour verification.

The PE sections are mapped at their virtual addresses (read-only copy of the preserved client binary), a
scratch heap and stack are added, and the function is called with the chosen convention. Calls that leave a
whitelist of addresses can be stubbed (return a fixed EAX and pop the right number of argument bytes), so a
pure function can be run without the rest of the client. Nothing here writes to the binary.

Library use:
    from emu import Emu
    e = Emu()
    buf = e.alloc(b"hello")
    eax = e.call(0x010dad20, args=[0, buf, 5], conv="cdecl")

CLI smoke test:  python -I emu.py selftest
"""
import struct, sys, zlib
from pathlib import Path
from unicorn import Uc, UcError, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE, UC_PROT_ALL
from unicorn.x86_const import (UC_X86_REG_EAX, UC_X86_REG_ECX, UC_X86_REG_EDX, UC_X86_REG_ESP, UC_X86_REG_EBP,
                               UC_X86_REG_EIP, UC_X86_REG_FS)

sys.path.insert(0, str(Path(__file__).resolve().parent))
from vslots import PE

STOP = 0x00001000          # return address: execution stops here
HEAP, HEAP_SIZE = 0x30000000, 0x01000000
STACK, STACK_SIZE = 0x20000000, 0x00100000
TEB = 0x7ffd0000


class Emu:
    def __init__(self, pe_path=PE):
        data = Path(pe_path).read_bytes()
        pe = struct.unpack_from("<I", data, 0x3C)[0]
        nsec, optsz = struct.unpack_from("<H", data, pe + 6)[0], struct.unpack_from("<H", data, pe + 20)[0]
        base = struct.unpack_from("<I", data, pe + 24 + 28)[0]
        self.mu = Uc(UC_ARCH_X86, UC_MODE_32)
        self.secs = []
        for i in range(nsec):
            s = pe + 24 + optsz + 40 * i
            vsize, vaddr, rsize, raw = struct.unpack_from("<IIII", data, s + 8)
            lo = base + vaddr
            size = (max(vsize, rsize) + 0xFFF) & ~0xFFF
            self.mu.mem_map(lo, size, UC_PROT_ALL)
            self.mu.mem_write(lo, data[raw:raw + min(rsize, vsize or rsize)])
            self.secs.append((lo, lo + size))
        self.mu.mem_map(STOP & ~0xFFF, 0x1000, UC_PROT_ALL)
        self.mu.mem_write(STOP, b"\xf4")                      # hlt
        self.mu.mem_map(STACK, STACK_SIZE, UC_PROT_ALL)
        self.mu.mem_map(HEAP, HEAP_SIZE, UC_PROT_ALL)
        self.mu.mem_map(TEB, 0x2000, UC_PROT_ALL)             # fs:[0] SEH chain, fs:[0x18] self
        self.mu.mem_write(TEB, struct.pack("<I", 0xFFFFFFFF))
        self.mu.mem_write(TEB + 0x18, struct.pack("<I", TEB))
        self._fs()
        self.heap_top = HEAP
        self.stubs = {}            # addr -> (eax, pop_bytes) or callable(emu) -> (eax, pop_bytes)
        self.trace_calls = []
        self.mu.hook_add(UC_HOOK_CODE, self._hook)

    def _fs(self):
        """Flat ring-0 data/stack segments plus an FS segment based at the fake TEB (for fs:[0] SEH frames)."""
        from unicorn.x86_const import UC_X86_REG_GDTR, UC_X86_REG_SS, UC_X86_REG_DS, UC_X86_REG_ES
        gdt = 0x7ffe0000
        self.mu.mem_map(gdt, 0x1000, UC_PROT_ALL)
        def desc(base, limit, access, flags):
            return struct.pack("<HHBBBB", limit & 0xFFFF, base & 0xFFFF, (base >> 16) & 0xFF, access,
                               ((limit >> 16) & 0xF) | (flags << 4), (base >> 24) & 0xFF)
        self.mu.mem_write(gdt + 8 * 1, desc(0, 0xFFFFF, 0x9A, 0xC))     # code, ring 0
        self.mu.mem_write(gdt + 8 * 2, desc(0, 0xFFFFF, 0x92, 0xC))     # flat data/stack, ring 0
        self.mu.mem_write(gdt + 8 * 3, desc(TEB, 0xFFF, 0x92, 0x4))     # FS -> TEB, ring 0
        self.mu.reg_write(UC_X86_REG_GDTR, (0, gdt, 0x1000, 0x0))
        for r in (UC_X86_REG_SS, UC_X86_REG_DS, UC_X86_REG_ES):
            self.mu.reg_write(r, 2 << 3)
        self.mu.reg_write(UC_X86_REG_FS, 3 << 3)

    def alloc(self, data=b"", size=None):
        size = size or max(len(data), 4)
        a = self.heap_top
        self.heap_top += (size + 15) & ~15
        if data:
            self.mu.mem_write(a, bytes(data))
        return a

    def read(self, addr, n):
        return bytes(self.mu.mem_read(addr, n))

    def u32(self, addr):
        return struct.unpack("<I", self.read(addr, 4))[0]

    def stub(self, addr, eax=0, pop=0):
        """Replace the function at addr: return eax and pop `pop` argument bytes (stdcall/thiscall `ret N`)."""
        self.stubs[addr] = (eax, pop)

    def _hook(self, mu, addr, size, _):
        if addr in self.stubs:
            s = self.stubs[addr]
            eax, pop = s(self) if callable(s) else s
            esp = mu.reg_read(UC_X86_REG_ESP)
            ret = struct.unpack("<I", mu.mem_read(esp, 4))[0]
            mu.reg_write(UC_X86_REG_EAX, eax & 0xFFFFFFFF)
            mu.reg_write(UC_X86_REG_ESP, esp + 4 + pop)
            mu.reg_write(UC_X86_REG_EIP, ret)
            self.trace_calls.append(addr)

    def call(self, addr, args=(), conv="cdecl", ecx=0, edx=0, max_insns=5_000_000):
        """conv: cdecl | stdcall | thiscall (ECX=this) | fastcall (ECX, EDX = first two args)."""
        args = list(args)
        if conv == "fastcall":
            ecx = args.pop(0) if args else 0
            edx = args.pop(0) if args else 0
        esp = STACK + STACK_SIZE - 0x1000
        for v in reversed(args):
            esp -= 4
            self.mu.mem_write(esp, struct.pack("<I", v & 0xFFFFFFFF))
        esp -= 4
        self.mu.mem_write(esp, struct.pack("<I", STOP))
        self.mu.reg_write(UC_X86_REG_ESP, esp)
        self.mu.reg_write(UC_X86_REG_EBP, esp)
        self.mu.reg_write(UC_X86_REG_ECX, ecx & 0xFFFFFFFF)
        self.mu.reg_write(UC_X86_REG_EDX, edx & 0xFFFFFFFF)
        self.mu.reg_write(UC_X86_REG_EAX, 0)
        try:
            self.mu.emu_start(addr, STOP, count=max_insns)
        except UcError as e:
            eip = self.mu.reg_read(UC_X86_REG_EIP)
            raise RuntimeError(f"emulation fault at {eip:08x}: {e}") from None
        return self.mu.reg_read(UC_X86_REG_EAX)


def selftest():
    """CRC at 010dad20 (zlib table 01ba7d68): crc(buf, len, seed) == zlib.crc32(seed as 4 LE bytes + buf),
    i.e. the SOE protocol's seeded CRC-32. Compared over random buffers and seeds."""
    import random
    e = Emu()
    rnd = random.Random(1)
    ok = total = 0
    for n in (0, 1, 7, 64, 1000):
        for seed in (0, 0x12345678, rnd.getrandbits(32)):
            buf = bytes(rnd.randrange(256) for _ in range(n))
            p = e.alloc(buf, size=max(n, 4))
            got = e.call(0x010dad20, args=[p, n, seed], conv="cdecl")
            want = zlib.crc32(struct.pack("<I", seed) + buf) & 0xFFFFFFFF
            total += 1; ok += got == want
            if got != want:
                print(f"DIFF len={n} seed={seed:08x} emu={got:08x} want={want:08x}")
    print(f"selftest crc 010dad20: {ok}/{total} match", "PASS" if ok == total else "FAIL")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "selftest":
        selftest()
