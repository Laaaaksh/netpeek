.PHONY: demo

# Converts a human-recorded screen capture into the README demo assets.
# The capture step itself can't be automated - see scripts/record-demo/README.md.
# Usage: make demo [INPUT=path/to/recording.mov]
demo:
	scripts/record-demo/convert.sh $(INPUT)
