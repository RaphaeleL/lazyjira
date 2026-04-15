.PHONY: cli 
cli: 
	cd cli && cargo run --release -- mine

.PHONY: tui
tui: 
	cd tui && cargo run --release	

.PHONY: clean 
clean: 
	rm -rf cli/target || true
	rm -rf tui/target || true

.PHONY: remove 
remove:
	rm $(HOME)/.local/bin/jira || true
	rm $(HOME)/.local/bin/lazyjira || true

.PHONY: install
prepare: remove
	mkdir -p $(HOME)/.local/bin/

.PHONY: build
build:
	cd cli && cargo build --release
	cd tui && cargo build --release

.PHONY: install
install: build prepare 
	cp $(CURDIR)/cli/target/release/lazyjira-cli $(HOME)/.local/bin/jira
	cp $(CURDIR)/tui/target/release/lazyjira-tui $(HOME)/.local/bin/lazyjira

.PHONY: link
link: build prepare 
	ln -sf $(CURDIR)/cli/target/release/lazyjira-cli $(HOME)/.local/bin/jira
	ln -sf $(CURDIR)/tui/target/release/lazyjira-tui $(HOME)/.local/bin/lazyjira
