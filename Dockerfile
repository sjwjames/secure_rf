FROM ubuntu:16.04
COPY ./ ./usr/src/rf
RUN apt-get update
RUN apt-get install -y vim
RUN apt-get install -y build-essential
RUn apt-get install -y manpages-dev
RUN apt-get install -y curl
RUN apt-get install -y python3
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain 1.37.0 -y
RUN echo "export PATH=~/.cargo/bin:$PATH" >> ~/.bashrc
RUN echo "export PS1='\u:\w$ '" >> ~/.bashrc
RUN chmod 777 /usr/src/rf/entrypoint.sh
ENTRYPOINT ["/usr/src/rf/entrypoint.sh"]
CMD ["run"]

