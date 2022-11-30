build:
	cargo build -p blockvisord
	cargo build -p babel --target x86_64-unknown-linux-musl

build-release:
	cargo build -p blockvisord --release
	strip target/release/bv
	strip target/release/blockvisord
	cargo build -p babel --target x86_64-unknown-linux-musl --release
	strip target/x86_64-unknown-linux-musl/release/babel

get-firecraker: FC_VERSION=v1.1.3
get-firecraker:
	rm -rf /tmp/fc
	mkdir -p /tmp/fc
	cd /tmp/fc; curl -L https://github.com/firecracker-microvm/firecracker/releases/download/${FC_VERSION}/firecracker-${FC_VERSION}-x86_64.tgz | tar -xz
	cp /tmp/fc/release-${FC_VERSION}-x86_64/firecracker-${FC_VERSION}-x86_64 /tmp/fc/firecracker
	cp /tmp/fc/release-${FC_VERSION}-x86_64/jailer-${FC_VERSION}-x86_64 /tmp/fc/jailer

bundle: get-firecraker build-release
	rm -rf /tmp/bundle.tar.gz
	rm -rf /tmp/bundle
	mkdir -p /tmp/bundle/blockvisor/bin /tmp/bundle/blockvisor/services
	cp target/release/bv /tmp/bundle/blockvisor/bin
	cp target/release/blockvisord /tmp/bundle/blockvisor/bin
	cp bv/data/tmux.service /tmp/bundle/blockvisor/services
	cp bv/data/blockvisor.service /tmp/bundle/blockvisor/services
	mkdir -p /tmp/bundle/babel/bin /tmp/bundle/babel/services
	cp target/x86_64-unknown-linux-musl/release/babel /tmp/bundle/babel/bin
	cp babel/data/babel.service /tmp/bundle/babel/services
	mkdir -p /tmp/bundle/firecracker/bin
	cp /tmp/fc/firecracker /tmp/bundle/firecracker/bin
	cp /tmp/fc/jailer /tmp/bundle/firecracker/bin
	tar -C /tmp -czvf /tmp/bundle.tar.gz bundle

tag: CARGO_VERSION = $(shell grep '^version' Cargo.toml | sed "s/ //g" | cut -d = -f 2 | sed "s/\"//g")
tag: GIT_VERSION = $(shell git describe --tags)
tag:
	@if [ "${CARGO_VERSION}" == "${GIT_VERSION}" ]; then echo "Version ${CARGO_VERSION} already tagged!"; \
	else git tag -a ${CARGO_VERSION} -m "Set version ${CARGO_VERSION}"; git push origin ${CARGO_VERSION}; \
	fi

install:
	install -m u=rwx,g=rx,o=rx target/debug/blockvisord /usr/bin/
	install -m u=rwx,g=rx,o=rx target/debug/bv /usr/bin/
	install -m u=rw,g=r,o=r bv/data/tmux.service /etc/systemd/system/
	install -m u=rw,g=r,o=r bv/data/blockvisor.service /etc/systemd/system/
	systemctl daemon-reload
	systemctl enable blockvisor.service
	for image in $$(find /root/.cache/blockvisor/images/ -name *.img); do \
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
