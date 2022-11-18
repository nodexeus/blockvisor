build:
	cargo build -p blockvisord
	cargo build -p babel --target x86_64-unknown-linux-musl

install:
	install -m u=rwx,g=rx,o=rx target/debug/blockvisord /usr/bin/
	install -m u=rwx,g=rx,o=rx target/debug/bv /usr/bin/
	install -m u=rw,g=r,o=r bv/data/tmux.service /etc/systemd/system/
	install -m u=rw,g=r,o=r bv/data/blockvisor.service /etc/systemd/system/
	systemctl daemon-reload
	systemctl enable blockvisor.service
	for image in $(wildcard /root/.cache/blockvisor/images/*.img); do \
		mount $$image /mnt/fc; \
		install -m u=rwx,g=rx,o=rx target/x86_64-unknown-linux-musl/debug/babel /mnt/fc/usr/bin/; \
		install -m u=rw,g=r,o=r babel/data/babel.service /mnt/fc/etc/systemd/system/; \
		install -m u=rw,g=r,o=r babel/protocols/helium/helium-validator.toml /mnt/fc/etc/babel.conf; \
		ln -s /mnt/fc/etc/systemd/system/babel.service /mnt/fc/etc/systemd/system/multi-user.target.wants/babel.service; \
		umount /mnt/fc; \
	done

reinstall:
	systemctl stop blockvisor.service
	make install
	systemctl start blockvisor.service
