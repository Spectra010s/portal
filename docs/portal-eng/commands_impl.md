# Commands Implementation (impl Commands)

The impl block is where the "behavior" of the application lives. While the Enum defines the choices, the implementation defines the actions.

## Key Method: execute()

This is the primary method called by main(). It takes the chosen command and brings it to life using a match statement.

**1. The Match Pattern**
The execute method uses an exhaustive match on self. This ensures that every possible subcommand is handled, preventing logical gaps.

**2. Pre-Flight Validation (The Send Logic)**
Before attempting any transfer, the Send variant performs a Reality Check:
 * file.exists(): We verify if the provided path actually exists on the disk.
 * Error Handling: If the file is missing, the program stops early and provides a clear error message to the user instead of crashing.
 
**3. Feedback Loop**
The implementation uses println! and the `.display()` method on PathBuf to provide the user with real-time status updates in the terminal.

## Responsibility:
To serve as the Brain of the operation. It handles the decision-making and ensures that the program behaves safely and predictably.

## Future Safety: The Quarantine Principle
To protect users from malicious files, the Portal "Receive" logic will implement a Quarantine Workflow:
 * Staging: Files are initially written to a restricted temporary directory.
 * Validation: The engine verifies that file metadata matches the actual content.
 * Promotion: Files are only moved to the final user-facing directory after a "Clean" status is confirmed.