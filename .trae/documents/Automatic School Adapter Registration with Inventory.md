To achieve the "derive-like" automatic registration you requested, I will create a procedural macro `#[register_school]`. This will allow you to simply annotate your school adapter struct, and the system will automatically handle the registration.

Here is the plan:

1.  **Create Macro Crate**:
    - Set up a new crate named `macros` inside the project to house the procedural macro (since proc-macros must reside in their own crate).
    - Configure the project as a workspace to include this new crate.

2.  **Implement `#[register_school]` Macro**:
    - Implement the logic to parse the annotated struct and generate the corresponding `inventory::submit!` code automatically.
    - This code will bridge the struct with the registration system without you writing manual boilerplate.

3.  **Setup `inventory` Registry**:
    - Add `inventory` dependency to the main crate.
    - Define `SchoolRegistration` struct in `src/adapters/traits.rs` to hold the factory functions.

4.  **Refactor Server State**:
    - Update `ServerState` to use `HashMap<String, Arc<dyn School>>` (using `String` keys for flexibility).
    - Modify `ServerState::from_config` to iterate over `inventory::iter::<SchoolRegistration>` and automatically instantiate all registered adapters.

5.  **Apply to Existing Adapters**:
    - Add `#[register_school]` to `NJUUndergradAdaptor` and `NJUGraduateAdapter`.
    - Remove the manual registration code.

This approach satisfies your "second best" requirement (using an attribute like `#[derive]`) and minimizes the risk of forgetting to register new adapters.
