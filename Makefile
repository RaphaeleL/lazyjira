INSTALL_DIR := $(HOME)/.local/bin
CLI_SCRIPT := $(CURDIR)/cli/jira.sh

.PHONY: install
prepare:
	mkdir -p $(INSTALL_DIR)
	chmod +x $(CLI_SCRIPT)

.PHONY: install-cli
install-cli: prepare 
	rm $(INSTALL_DIR)/jira
	cp $(CLI_SCRIPT) $(INSTALL_DIR)/jira

.PHONY: link-cli
link-cli: prepare 
	rm $(INSTALL_DIR)/jira
	ln -sf $(CLI_SCRIPT) $(INSTALL_DIR)/jira
