SHADERS_DIR = shaders
SHADERS_BUILD_DIR = $(SHADERS_DIR)/build

SPV_EXTENSION = .spv

VERTEX_SHADER_FILE_NAME = shader.vert
VERTEX_SHADER = $(SHADERS_DIR)/$(VERTEX_SHADER_FILE_NAME)
VERTEX_SHADER_SPV = $(SHADERS_BUILD_DIR)/$(VERTEX_SHADER_FILE_NAME)$(SPV_EXTENSION)

FRAGMENT_SHADER_FILE_NAME = shader.frag
FRAGMENT_SHADER = $(SHADERS_DIR)/$(FRAGMENT_SHADER_FILE_NAME)
FRAGMENT_SHADER_SPV = $(SHADERS_BUILD_DIR)/$(FRAGMENT_SHADER_FILE_NAME)$(SPV_EXTENSION)

GLSLC = glslc

all: compile_shaders
	cargo b --release
.PHONE: all

run: compile_shaders
	cargo r --release
.PHONY: run

run_debug: compile_shaders
	RUST_BACKTRACE=1 cargo r --features validation_layers
.PHONY: run_debug

debug: compile_shaders
	cargo b --features validation_layers
.PHONY: debug

clean:
	rm -rf $(SHADERS_BUILD_DIR)
	rm -rf target/
.PHONY: clean

fclean: clean
.PHONY: fclean

re: fclean
	$(MAKE) all
.PHONY: re

compile_shaders: $(FRAGMENT_SHADER_SPV) $(VERTEX_SHADER_SPV)
.PHONY: compile_shaders

$(FRAGMENT_SHADER_SPV): $(FRAGMENT_SHADER)
	@mkdir -p $(shell dirname $(FRAGMENT_SHADER_SPV))
	$(GLSLC) $(FRAGMENT_SHADER) -o $(FRAGMENT_SHADER_SPV)

$(VERTEX_SHADER_SPV): $(VERTEX_SHADER)
	@mkdir -p $(shell dirname $(VERTEX_SHADER_SPV))
	$(GLSLC) $(VERTEX_SHADER) -o $(VERTEX_SHADER_SPV)
