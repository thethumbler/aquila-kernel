use prelude::*;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_set::BTreeSet;

extern "C" {
    static __modules_start: u8;
    static __modules_end: u8;
}

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Module {
    name: &'static str,
    deps: Option<fn() -> Vec<&'static str>>,
    init: Option<fn() -> Result<(), Error>>,
    fini: Option<fn() -> Result<(), Error>>,
}

fn load_with_deps(done: &mut BTreeSet<&'static str>, pending: &mut BTreeMap<&'static str, Module>) {
    if pending.is_empty() {
        return;
    }

    let (name, module) = pending.pop_first().unwrap();
    let deps = module.deps.map(|f| f().into_iter().collect::<BTreeSet<&str>>()).unwrap_or(BTreeSet::new());
    let mut loaded = false;
    
    if deps.is_subset(&done) {
        module.init.map(|f| f()).unwrap_or(Ok(()));
        done.insert(name);
        loaded = true;
    }

    load_with_deps(done, pending);

    if loaded {
        return;
    }

    if deps.is_subset(&done) {
        module.init.map(|f| f()).unwrap_or(Ok(()));
        done.insert(name);
    } else {
        panic!("failed to load module: {}", name);
    }
}

pub fn init() -> isize {
    unsafe {
        print!("kernel: loading builtin modules\n");

        /* initalize built-in modules */
        let modules_start = &__modules_start as *const _ as usize;
        let modules_end   = &__modules_end   as *const _ as usize;
        let modules_ptr   = &__modules_start as *const _ as *const Module;

        let nr = (modules_end - modules_start) / core::mem::size_of::<Module>();
        let modules = core::slice::from_raw_parts(modules_ptr, nr);

        print!("kernel: found {} modules\n", nr);

        let mut pending = modules.iter().map(|module| (module.name, module.clone())).collect::<BTreeMap<&'static str, Module>>();
        let mut done = BTreeSet::new();

        load_with_deps(&mut done, &mut pending);

        return 0;
    }
}

pub macro module_define {
    ($name:expr, $deps:expr, $init:expr, $fini:expr) => {
        #[used]
        #[link_section = ".module"]
        static __MODULE__: Module = Module {
            name: $name,
            deps: $deps,
            init: $init,
            fini: $fini,
        };
    }
}
