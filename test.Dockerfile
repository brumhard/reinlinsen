FROM busybox as build

COPY src/main.rs main.rs
RUN mkdir testing
RUN echo "test" > testing/testfile
RUN echo "lel" > testing/lelfile
RUN rm testing/testfile
RUN echo "else" > lelfile
RUN rm -rf testing