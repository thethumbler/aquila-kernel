use prelude::*;

use crate::include::bits::errno::*;
use crate::include::fs::vfs::FileDescriptor;
use crate::include::net::socket::socklen_t;
use crate::include::net::socket::SocketAddress;

pub unsafe fn socket_create(file: *mut FileDescriptor, domain: isize, _type: isize, protocol: isize) -> isize {
    /*
    switch (domain) {
        case AF_UNIX:
            return socket_unix_create(file, domain, type, protocol);
    }
    */

    return -EAFNOSUPPORT;
}

pub unsafe fn socket_accept(file: *mut FileDescriptor, conn: *mut FileDescriptor, addr: *const SocketAddress, len: socklen_t) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->accept)
        return -EOPNOTSUPP;

    return file->socket->ops->accept(file, conn, addr, len);
    */

    return -EINVAL;
}

pub unsafe fn socket_bind(file: *mut FileDescriptor, addr: *const SocketAddress, len: usize) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->bind)
        return -EOPNOTSUPP;

    return file->socket->ops->bind(file, addr, len);
    */

    return -EINVAL;
}

pub unsafe fn socket_connect(file: *mut FileDescriptor, addr: *const SocketAddress, len: usize) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->connect)
        return -EOPNOTSUPP;

    return file->socket->ops->connect(file, addr, len);
    */

    return -EINVAL;
}

pub unsafe fn socket_listen(file: *mut FileDescriptor, backlog: isize) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->listen)
        return -EOPNOTSUPP;

    return file->socket->ops->listen(file, backlog);
    */

    return -EINVAL;
}

pub unsafe fn socket_send(file: *mut FileDescriptor, buf: *const u8, len: usize, flags: isize) -> isize {
    /*
    if (!file)
        return -EINVAL;

    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!(file->socket))
        return -EINVAL;

    if (!file->socket->ops || !file->socket->ops->send)
        return -EOPNOTSUPP;

    return file->socket->ops->send(file, buf, len, flags);
    */

    return -EINVAL;
}

pub unsafe fn socket_recv(file: *mut FileDescriptor, buf: *mut u8, len: usize, flags: isize) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->recv)
        return -EOPNOTSUPP;

    return file->socket->ops->recv(file, buf, len, flags);
    */

    return -EINVAL;
}

pub unsafe fn socket_can_read(file: *mut FileDescriptor, len: usize) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->can_read)
        return -EOPNOTSUPP;

    return file->socket->ops->can_read(file, len);
    */

    return -EINVAL;
}

pub unsafe fn socket_can_write(file: *mut FileDescriptor, len: usize) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->can_write)
        return -EOPNOTSUPP;

    return file->socket->ops->can_write(file, len);
    */

    return -EINVAL;
}

pub unsafe fn socket_shutdown(file: *mut FileDescriptor, how: isize) -> isize {
    /*
    if (!(file->flags & FILE_SOCKET))
        return -ENOTSOCK;

    if (!file->socket->ops || !file->socket->ops->shutdown)
        return -EOPNOTSUPP;

    file->socket->ref--;

    if (!file->socket->ref)
        return file->socket->ops->shutdown(file, how);

    return 0;
    */

    return -EINVAL;
}
