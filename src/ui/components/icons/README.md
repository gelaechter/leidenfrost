# Icons

The icons in leidenfrost have been arranged using [icomoon](https://icomoon.io/).

The three files in this directory serve the following purpose:

1. `leidenfrost.json` is the icomoon project file which can be imported to make changes to the icon pack
2. `leidenfrost.icomoon.json` are the icon definitions containing metadata like the icon names and their corresponding codepoints
3. `leidenfrost.ttf` is a font which displays the corresponding codepoints as the given icons

The `build.rs` build-script reads the icon definitions in `leidenfrost.icomoon.json` and generates `src/ui/components/icons.rs` based on that,
to provide an easy interface for displaying the icons.

# Password

Iced's TextInput fields don't allow us to mask the password, because they resend their current value.
The value gets written back through the message. Meaning, code like this:

```rust
widget::text_input(
    "Password",
    // Mask password
    &self.password.chars().map(|_| '*').collect::<String>(),
)
.on_input(Cmd::ChangePassword)
```

will overwrite the output when typing the password `thisismypassword`

```rust
[src/ui/router/settings/endpoint.rs:248:17] &password = "t"
[src/ui/router/settings/endpoint.rs:248:17] &password = "*h"
[src/ui/router/settings/endpoint.rs:248:17] &password = "**i"
[src/ui/router/settings/endpoint.rs:248:17] &password = "***s"
[src/ui/router/settings/endpoint.rs:248:17] &password = "****i"
[src/ui/router/settings/endpoint.rs:248:17] &password = "*****s"
[src/ui/router/settings/endpoint.rs:248:17] &password = "******m"
[src/ui/router/settings/endpoint.rs:248:17] &password = "*******y"
[src/ui/router/settings/endpoint.rs:248:17] &password = "********p"
[src/ui/router/settings/endpoint.rs:248:17] &password = "*********a"
[src/ui/router/settings/endpoint.rs:248:17] &password = "**********s"
[src/ui/router/settings/endpoint.rs:248:17] &password = "***********s"
[src/ui/router/settings/endpoint.rs:248:17] &password = "************w"
[src/ui/router/settings/endpoint.rs:248:17] &password = "*************o"
[src/ui/router/settings/endpoint.rs:248:17] &password = "**************r"
[src/ui/router/settings/endpoint.rs:248:17] &password = "***************d"
```
The simplest fix is simply hiding the password by using a [password font](https://github.com/RouninMedia/password-font),
which is now in this directory under `password.ttf`