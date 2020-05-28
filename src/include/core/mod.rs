pub mod arch;
pub mod string;
pub mod types;
pub mod time;
pub mod module;

//pub mod logger {
//    #[repr(C)]
//    pub enum LogLevel {
//        None: usize     = 0,
//        Emergency: usize    = 1,
//        Alert: usize    = 2,
//        Crititcal: usize     = 3,
//        Error: usize      = 4,
//        Warning: usize  = 5,
//        Notice: usize   = 6,
//        Info: usize     = 7,
//        Debug: usize    = 8,
//    }
//
//    //int vprintk(const char *fmt, va_list args);
//    //int printk(const char *fmt, ...);
//
//    //#define LOGGER_DEFINE(module, name, _level) \
//    //int name(int level, const char *fmt, ...) \
//    //{ \
//    //    if (level <= _level) { \
//    //        va_list args; \
//    //        va_start(args, fmt); \
//    //        printk("%s: ", #module); \
//    //        vprintk(fmt, args); \
//    //        va_end(args); \
//    //    } \
//    //    return 0; \
//    //}
//
//    //#define LOGGER_DECLARE(name) \
//    //int name(int level, const char *fmt, ...);
//}
