public class HeliumLogStream extends java.io.OutputStream {
    public native void write(int b);
    public native void write(byte[] b, int off, int len);
    public void flush() {}
    public void close() {}
}
