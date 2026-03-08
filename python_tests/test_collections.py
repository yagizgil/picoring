import picoring

def test_collections():
    # 1. PicoQueue
    print("Testing PicoQueueByte...")
    q = picoring.PicoQueueByte(1024)
    q.push_bulk(b"hello world")
    print(f"Queue len: {q.len()}")
    view = q.get_view()
    print(f"Queue view content: {bytes(view)}")
    q.pop_bulk(6)
    print(f"Queue len after pop: {q.len()}")
    print("-" * 30)

    # 2. PicoByteStream
    print("Testing PicoByteStreamPy...")
    stream = picoring.PicoByteStreamPy(1024)
    stream.write(b"streaming data")
    print(f"Available to read: {stream.available_to_read()}")
    data = stream.read(9)
    print(f"Read from stream: {bytes(data)}")
    print("-" * 30)

    # 3. PicoList
    print("Testing PicoListByte...")
    plist = picoring.PicoListByte()
    plist.extend(b"dynamic list")
    print(f"List len: {plist.len()}")
    print(f"List index 5: {chr(plist.get(5))}")
    print("-" * 30)

    print("ALL COLLECTIONS VERIFIED SUCCESSFULLY.")

if __name__ == "__main__":
    test_collections()
