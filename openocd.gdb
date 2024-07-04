target extended-remote :3333

monitor arm semihosting enable

file target/thumbv7em-none-eabihf/release/app

load

continue
