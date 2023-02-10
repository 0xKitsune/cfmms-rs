ifndef ETHEREUM_MAINNET_ENDPOINT
    ifneq (,$(wildcard ./.env))
        include .env
        export
    else
        export ETHEREUM_MAINNET_ENDPOINT=https://eth.llamarpc.com
    endif
endif

EXAMPLES=$(wildcard examples/*.rs)
EXAMPLES_BIN=$(patsubst examples/%.rs,%,$(EXAMPLES))

define generate_example_target
$(1):
	cargo run --example $(1)
endef

$(foreach example,$(EXAMPLES_BIN),$(eval $(call generate_example_target,$(example))))

.PHONY: all
all: $(EXAMPLES_BIN)
