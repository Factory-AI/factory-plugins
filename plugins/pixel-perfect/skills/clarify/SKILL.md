---
name: clarify
description: "Improve unclear UX copy, error messages, labels, and instructions by reading the actual rendered text in the browser. Use when the user mentions confusing text, unclear labels, bad error messages, or wants better UX writing."
user-invocable: true
---

Find and fix unclear interface text by reading what users actually see in the browser.

## 1. Read Rendered Text via agent-browser

Don't just grep source code—use agent-browser to see what users see:

- Navigate to each screen in the target area
- Read the actual rendered text: headings, labels, buttons, tooltips, error states
- Trigger error states to see real error messages
- Check empty states, loading states, and confirmation dialogs
- Note text that's truncated, overlapping, or visually confusing in context

This catches issues that code review misses: dynamic text, interpolated strings, text that only appears in specific states.

## 2. Identify Copy Problems

Flag text that fails these checks:

### Vague Error Messages
- ❌ "Something went wrong" / "Error" / "Invalid input"
- ✅ "We couldn't save your changes. Check your connection and try again."
- ✅ "Email address needs an @ symbol. Example: name@company.com"

### Confusing Labels
- ❌ Generic: "Value", "Type", "Status", "Items"
- ❌ Jargon: "Webhook payload", "OAuth scope", "Idempotency key"
- ✅ Specific: "Monthly revenue", "Account type", "Delivery status"

### Wordy Instructions
- ❌ "In order to proceed with the creation of your account, please fill in the required fields below"
- ✅ "Create your account"

### Weak CTAs
- ❌ "Submit" / "OK" / "Click here" / "Yes"
- ✅ "Save changes" / "Create project" / "Delete account"

### Passive Voice
- ❌ "Your file has been uploaded"
- ✅ "File uploaded" or "We uploaded your file"

### Missing Context
- ❌ Empty state: "No items"
- ✅ Empty state: "No projects yet. Create one to get started."

## 3. Apply Fixes

For each identified issue:

- **Replace jargon** with plain language the target audience understands
- **Make CTAs action-oriented**: verb + noun ("Save draft", "Send invite")
- **Error messages**: explain what happened + how to fix it
- **Be specific**: "Your email" not "Enter value", "3 team members" not "Items"
- **Be concise**: cut unnecessary words but keep clarity
- **Be consistent**: same term everywhere (don't alternate "delete"/"remove"/"discard")

### Priority Order
1. Error messages (highest user frustration)
2. Primary CTAs (directly affect task completion)
3. Form labels and help text
4. Empty states and loading messages
5. Navigation labels
6. Success messages and confirmations

## 4. Verify in Context

Use agent-browser to confirm fixes work in the actual UI:

- Read the updated text as rendered in the browser
- Check that improved copy fits the layout (no overflow, no truncation)
- Trigger error states again to verify error messages are helpful
- Walk through complete user flows to ensure copy is consistent
- Verify placeholder text disappears correctly and labels remain visible

## Rules

- Never use humor in error messages—be empathetic and helpful
- Don't blame the user ("You entered an invalid email" → "This email format isn't recognized")
- Match the audience's vocabulary (technical docs can use technical terms; consumer UI cannot)
- Maintain consistent terminology throughout the entire application
- Don't change microcopy that has legal or compliance implications without flagging it
- Run lint, type-check, and tests after changes
