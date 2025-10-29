Run a code review on the current diff.
Check for:
-  unnecessary or duplicate code
-  illtyped code or rust gotchas, footguns or rookie mistakes
-  unnecessary comments or bloat
-  overly complicated logic that can be done simpler
-  unintuitive or inconsistent naming
-  inconsistent coding patterns
-  fighting systems (systems that conflict or work against each other, duplicate functionality, or competing state updates)
Outline each issue. Only look at new additions and the current diff. Suggest how you would improve it. Present a bunch of suggestion as a diff. Don't edit anything yet. Consult the oracle for difficult things.
