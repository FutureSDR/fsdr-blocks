structure.png:
	cargo structure --local -m | dot -Tpng > structure.png

check:
	./check.sh

.PHONY: structure.png