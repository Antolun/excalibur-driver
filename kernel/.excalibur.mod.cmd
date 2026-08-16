savedcmd_excalibur.mod := printf '%s\n'   excalibur_main.o excalibur_core_rust.o | awk '!x[$$0]++ { print("./"$$0) }' > excalibur.mod
