This entire project is being coded via Claude Sonnet 4 in Claude Code. The initial primordial documents (Implementor prompts, structure, claude.md, and integration_guide) were made via Opus 4.1 in the app. 

Example learning points:
- Claude can reference the VS Code problems pane, correct faults, but will not recognize the pane is not updated, and so recursively sees no issue but checks the pane
- Claude and the Rust-Analyzer see different sets of issues with the code base, and so using both caught essentially all inherent defects, but results in copious unused items (and no documentation)
- The system was able to successfully implement every component to a runtime state but main loop error caused immediate exit
- Agent design requires first directing claude to perform actions and seeing what mistakes are made, especially needlessly adding content to context when it is not necessary
- Claude will bottle itself to rigid structures, making behaviour predictable, but requiring some review phase where the current implementation is analyzed and suggested changes to structure are made