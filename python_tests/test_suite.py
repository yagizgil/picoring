import unittest
import picoring

class TestPicoCollections(unittest.TestCase):
    def test_ring_buffer(self):
        ring = picoring.PicoRingByte(1024)
        ring.push_bytes(b"test")
        self.assertEqual(ring.len(), 4)
        view = ring.get_readable_view()
        self.assertEqual(bytes(view), b"test")

    def test_queue(self):
        q = picoring.PicoQueueByte(1024)
        q.push_bulk(b"queue_test")
        self.assertEqual(q.pop_bulk(5), b"queue")
        self.assertEqual(q.len(), 5)

    def test_byte_stream(self):
        stream = picoring.PicoByteStreamPy(1024)
        stream.write(b"stream_test")
        self.assertEqual(stream.read(6), b"stream")
        self.assertEqual(stream.available_to_read(), 5)

    def test_list(self):
        plist = picoring.PicoListByte()
        plist.extend(b"list_test")
        self.assertEqual(plist.len(), 9)
        self.assertEqual(plist.get(0), ord('l'))

if __name__ == "__main__":
    unittest.main()
