# aptos_clockwork
Works like a clockwork on the Aptos blockchain.

=====================

<img width="827" height="547" alt="Screenshot-2025-11-21-13-58-06" src="https://github.com/user-attachments/assets/90777e78-4523-41c2-ba75-d65f25d1052b" />


> Installation

`sudo mkdir /opt`

`sudo chown -R <USER> /opt/`

> We set dependencies for the node.

`sudo pkg install rust gmake gcc hidapi rocksdb`

> Download the program

`cd /opt`

`git clone https://github.com/Clockwork6400/aptos_clockwork.git`

`cd /opt/aptos_clockwork`

`cargo build`

> Expand node

`cd /opt`

`git clone https://github.com/aptos-labs/aptos-core.git`

`cd aptos-core/`

`cargo build --package aptos --profile cli`

> Launch the program

`./target/debug/aptos_clockwork`

> Don't forget to make windows floating for tiled WM (if necessary).

=====================
> How to get premium: donate more than 1 apt or read the program code (requires IQ 120).

What premium gives you:

- the ability to change the program theme

- the ability to rotate your private key

- something else

=====================

Minimal yaml file:

cat aptos_clockwork.yaml 
```
{
  use_awk: "no", 
  premium: "no",
  theme: "no"
}
```
