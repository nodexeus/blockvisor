build:
	cargo build

install:
	install -m u=rwx,g=rx,o=rx target/debug/blockvisord /usr/bin/
	install -m u=rwx,g=rx,o=rx target/debug/bv /usr/bin/
	install -m u=rw,g=r,o=r data/blockvisor.service /etc/systemd/system/
	install -m u=rw,g=r,o=r data/com.BlockJoy.blockvisor.conf /etc/dbus-1/system.d/
	install -m u=rw,g=r,o=r data/babel-bus.conf /etc/
	install -m u=rw,g=r,o=r data/babel-bus.service /etc/systemd/system/
	install -m u=rw,g=r,o=r data/babel-bus.socket /etc/systemd/system/
	systemctl daemon-reload

reinstall:
	bv stop || true
	make install
	bv start
