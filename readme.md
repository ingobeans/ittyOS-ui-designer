# ittyOS ui designer

a rust tool to design UI screens that transpile to C code.

use `ittyOS-ui-designer <target_file_name>`, ex `ittyOS-ui-designer gameUI.c`, to create that ui file.

it will also generate a header file. when editing that UI in the future, simply use the same command on the existing file.

currently, the program displays a representation of the UI, and live updates the c code when the config is changed. (buggy).

todo:
* fix ordering
* fix buggy live update