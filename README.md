# prickly

Cross-platform TUI application for editing prc files.

## Installation:

Install to PATH automatically with cargo:

`cargo install --git https://github.com/BenHall-7/prickly.git --branch main`

If you prefer to choose the installation folder, you can clone and build the repository with the same link, then move the application to a folder of your choice.

## Startup:

Possible options:

- Drag and drop a param file onto the prickly executable
- Set prickly to be the default program for .prc files
- Specify the file to open from the terminal in the app arguments
- Open the application and load the file manually with the file explorer

[Param labels](https://github.com/ultimate-research/param-labels) are loaded by precedence:

1. If there is a ParamLabels.csv file in the current directory
2. If there is a ParamLabels.csv file in the application directory
  - If installed with `cargo install`, find the `.cargo/bin` directory

## Command shortcuts:

- `Ctrl + O`: open the file explorer for opening files
- `Ctrl + S`: open the file explorer for saving files
- `Enter`: Enter or edit a param node, or open a folder/file in the file explorer
- `Backspace`: Go to the parent param, or to the parent folder in the file explorer
- `Esc`: Exit the given prompt (param viewer, file explorer, inputs, etc)
- `/`: Begin typing a filter for params, or search for a file in the file explorer
  - NOTE: If the file explorer is in saving mode, pressing `Enter` will attempt to save with the given name
- `+`: Increment a value by 1 (integers and floats). Bools are inverted.
- `-`: Decrement a value by 1 (integers and floats). Bools are inverted.
