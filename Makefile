.PHONY: demo

# Builds, records, and converts the README demo assets in one go.
# Usage: make demo [DURATION=65]
demo:
	scripts/record-demo/capture.sh $(DURATION)
