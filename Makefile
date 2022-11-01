build:
	cargo build -p blockvisord
	cargo build -p babel --target x86_64-unknown-linux-musl

install:
	install -m u=rwx,g=rx,o=rx target/debug/blockvisord /usr/bin/
	install -m u=rwx,g=rx,o=rx target/debug/bv /usr/bin/
	install -m u=rw,g=r,o=r bv/data/tmux.service /etc/systemd/system/
	install -m u=rw,g=r,o=r bv/data/blockvisor.service /etc/systemd/system/
	install -m u=rw,g=r,o=r bv/data/com.BlockJoy.blockvisor.conf /etc/dbus-1/system.d/
	install -m u=rw,g=r,o=r bv/data/babel-bus.conf /etc/
	install -m u=rw,g=r,o=r bv/data/babel-bus.service /etc/systemd/system/
	install -m u=rw,g=r,o=r bv/data/babel-bus.socket /etc/systemd/system/
	systemctl daemon-reload
	systemctl enable blockvisor.service
	mount /var/lib/blockvisor/debian.ext4 /mnt/fc
	install -m u=rwx,g=rx,o=rx target/x86_64-unknown-linux-musl/debug/babel /mnt/fc/usr/bin/
	install -m u=rw,g=r,o=r babel/data/babel.service /mnt/fc/etc/systemd/system/
	install -m u=rw,g=r,o=r babel/examples/helium.toml /mnt/fc/etc/babel.conf
	umount /mnt/fc

reinstall:
	systemctl stop blockvisor.service
	make install
	systemctl start blockvisor.service
