use bignum::fields::U256::U256;

struct State {
    state_root: [u8; 32],
    command_root: [u8; 32],
    current_sig: [u8; 32 * 512],
    current_pubkey: [u8; 32],
    miner: [u8; 32],
    current_subcommand: u32,
}
struct Opening {
    hashes: [[u8; 32]; 256],
}
impl Opening {
    fn patch(self: Self, path: [u8; 32], mut node: [u8; 32]) -> [u8; 32] {
        for j in 0..256 {
            let mut msg = [0u8; 64];
            if (path[j >> 3] >> (j & 7) as u8) & 1 == 0 {
                for i in 0..32 {
                    msg[i] = node[i]
                }
                for i in 0..32 {
                    msg[i + 32] = self.hashes[j][i]
                }
            } else {
                for i in 0..32 {
                    msg[i + 32] = node[i]
                }
                for i in 0..32 {
                    msg[i] = self.hashes[j][i]
                }
            }
            node = sha256::digest(msg);
        }
        node
    }
    fn open(self: Self, root: [u8; 32], path: [u8; 32], node: [u8; 32]) {
        assert(self.patch(path, node) == root);
    }
}
fn main(
    mut root: pub State,
    kind_opening: Opening,
    kind: [u8; 32],
    arg_opening: Opening,
    arg: [u8; 32],
    xarg_opening: Opening,
    xarg: [u8; 32],
    xarg2: [u8; 32],
    arg_ptr_opening1: Opening,
    mut new_hash: [u8; 32 * 512],
) -> pub State {
    let mut is_eob: bool = true;
    for i in 0..32 {
        if root.command_root[i] != 0 {
            is_eob = false
        }
    }
    if is_eob {
        for j in 0..512 {
            if (xarg[j >> 4] >> ((j >> 1) & 7) as u8) & 1 != (j & 1) as u8 {
                for i in 0..32 {
                    new_hash[j * 32 + i] = root.current_sig[j * 32 + i]
                }
            } else {
                let mut x = [0u8; 32];
                for i in 0..32 {
                    x[i] = root.current_sig[j * 32 + i]
                }
                x = sha256::digest(x);
                for i in 0..32 {
                    new_hash[j * 32 + i] = x[i]
                }
            }
        }
        let hash = sha256::digest(new_hash);
        if hash == root.current_pubkey {
            root.command_root = xarg;
            root.current_pubkey = sha256::digest(root.current_pubkey);
            root
        } else {
            for i in 0..32 {
                assert((root.state_root[i >> 3] >> ((i & 7) as u8)) & 1 == 0);
            }
            for i in 0..(32 * 512) {
                root.current_sig[i] = 0xff;
            }
            root
        }
    } else {
        let mut kindx = kind;
        kindx[31] = 0xff;
        kind_opening.open(root.command_root, [0u8; 32], kindx);
        arg_opening.open(root.command_root, kindx, arg);
        if kind[0] == 0 {
            assert(root.current_subcommand == 0);
            root.command_root = [0u8; 32];
            arg_ptr_opening1.open(root.state_root, arg, sha256::digest(new_hash));
            root.current_sig = new_hash;
            root.current_pubkey = xarg2;
        } else {
            if root.current_subcommand == 0 {
                root.current_pubkey = sha256::digest(root.current_pubkey);
                root.current_subcommand = kind[0] as u32;
                arg_ptr_opening1.open(root.command_root, root.current_pubkey, kind);
                xarg_opening.open(root.command_root, kind, xarg);
            } else {
                arg_ptr_opening1.open(root.command_root, root.current_pubkey, kind);
                xarg_opening.open(root.command_root, kind, xarg);
            }
        }
        root
    }
}
