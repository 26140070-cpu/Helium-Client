public class HeliumLogStream extends java.io.OutputStream {
    public int kind;
    public HeliumLogStream(int kind) { this.kind = kind; }
    public HeliumLogStream() { this.kind = 0; }
    public native void write(int b);
    public native void write(byte[] b, int off, int len);
    public void flush() {}
    public void close() {}
}
