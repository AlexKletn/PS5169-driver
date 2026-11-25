from pathlib import Path
import sys
import tomllib


SEQUENCES = (
    "initial",
    "usb_enable",
    "usb_disable",
    "dp_enable",
    "dp_disable",
    "mixed_enable",
    "mixed_disable",
)


def generate(profile):
    if set(profile) != set(SEQUENCES):
        raise ValueError("expected exactly these sequences: " + ", ".join(SEQUENCES))
    lines = [
        "pub const SELECTED_SEQUENCES: Option<RegisterSequences<&'static [u8]>> =",
        "    Some(RegisterSequences {",
    ]
    for name in SEQUENCES:
        sequence = profile[name]
        if not isinstance(sequence, list) or not sequence:
            raise ValueError(f"{name} must be a nonempty array")
        for pair in sequence:
            if not isinstance(pair, list) or len(pair) != 2:
                raise ValueError(f"{name} must contain register/value pairs")
            if any(type(byte) is not int or not 0 <= byte <= 255 for byte in pair):
                raise ValueError(f"{name} registers and values must be bytes")
        if name == "initial" and not any(address == 0x51 for address, _ in sequence):
            raise ValueError("initial must include USB RX equalization register 0x51")
        values = ", ".join(f"0x{byte:02x}" for pair in sequence for byte in pair)
        lines.append(f"        {name}: &[{values}],")
    lines.append("    });")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    source, output = sys.argv[1:]
    try:
        with Path(source).open("rb") as profile_file:
            rust = generate(tomllib.load(profile_file))
        Path(output).write_text(rust)
    except (OSError, ValueError) as error:
        sys.exit(f"{source}: {error}")
