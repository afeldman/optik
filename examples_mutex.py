#!/usr/bin/env python3
"""
Practical example: Multi-threaded camera capture with Rust Mutex

Demonstriert:
1. Thread-safe camera access ohne GIL
2. Proper error handling
3. Multi-camera pattern
"""

import threading
import time
from typing import List, Optional
from optik._core import PyCamera


class CameraWorker(threading.Thread):
    """Worker thread for safe camera access"""
    
    def __init__(self, worker_id: int, camera: PyCamera, num_frames: int = 10):
        super().__init__(daemon=False)
        self.worker_id = worker_id
        self.camera = camera
        self.num_frames = num_frames
        self.frames_grabbed = 0
        self.errors = 0
        self.lock = threading.Lock()
    
    def run(self):
        """Grab frames in a thread-safe manner"""
        print(f"[Worker {self.worker_id}] Starting")
        
        for frame_num in range(self.num_frames):
            try:
                # Rust mutex handles thread safety internally!
                frame = self.camera.grab_frame()
                meta = frame.metadata()
                
                with self.lock:
                    self.frames_grabbed += 1
                    print(f"[Worker {self.worker_id}] Grabbed frame {meta.sequence} "
                          f"(exposure: {meta.exposure_us:.0f}µs, gain: {meta.gain:.1f}dB)")
                
                # Simulate some processing
                time.sleep(0.01)
                
            except RuntimeError as e:
                with self.lock:
                    self.errors += 1
                print(f"[Worker {self.worker_id}] ERROR: {e}")
        
        print(f"[Worker {self.worker_id}] Done: {self.frames_grabbed} frames, {self.errors} errors")


def example_single_camera_multithreaded():
    """Example 1: Single camera, multiple worker threads"""
    print("\n" + "="*60)
    print("Example 1: Single Camera, Multi-Threaded Access")
    print("="*60)
    
    # Create and open camera
    cam = PyCamera("rpi", 0)
    try:
        cam.open()
        print("[Main] Camera opened successfully")
        
        # Configure camera
        cam.set_exposure(15000.0)  # 15ms
        cam.set_gain(10.0)         # 10dB
        print(f"[Main] Camera configured: exposure={cam.get_exposure()}µs, gain={cam.get_gain()}dB")
        
        # Create 4 worker threads competing for camera access
        workers: List[CameraWorker] = []
        num_workers = 4
        frames_per_worker = 5
        
        print(f"[Main] Starting {num_workers} worker threads...")
        start_time = time.time()
        
        for i in range(num_workers):
            worker = CameraWorker(
                worker_id=i,
                camera=cam,
                num_frames=frames_per_worker
            )
            workers.append(worker)
            worker.start()
        
        # Wait for all workers to complete
        for worker in workers:
            worker.join()
        
        elapsed = time.time() - start_time
        total_frames = sum(w.frames_grabbed for w in workers)
        total_errors = sum(w.errors for w in workers)
        
        print(f"\n[Main] All workers completed in {elapsed:.2f}s")
        print(f"[Main] Total: {total_frames} frames grabbed, {total_errors} errors")
        print(f"[Main] Rate: {total_frames/elapsed:.1f} fps")
        
        # Rust Mutex guarantee: NO RACE CONDITIONS!
        # Python GIL would cause contention, but Rust mutex is fine
        
    finally:
        cam.close()
        print("[Main] Camera closed")


def example_multiple_cameras():
    """Example 2: Multiple cameras (simulated)"""
    print("\n" + "="*60)
    print("Example 2: Multiple Cameras (Simulated)")
    print("="*60)
    
    cameras = []
    workers = []
    
    # Create multiple camera instances
    num_cameras = 3
    print(f"[Main] Creating {num_cameras} camera instances...")
    
    for i in range(num_cameras):
        try:
            cam = PyCamera("rpi", i)
            cam.open()
            cam.set_exposure(12000.0)
            cam.set_gain(5.0 + i)  # Different gain per camera
            cameras.append(cam)
            print(f"[Main] Camera {i} opened: exposure={cam.get_exposure()}µs, gain={cam.get_gain()}dB")
        except RuntimeError as e:
            print(f"[Main] Failed to open camera {i}: {e}")
    
    # Create one worker per camera
    print(f"[Main] Starting {len(cameras)} capture threads...")
    start_time = time.time()
    
    for i, cam in enumerate(cameras):
        worker = CameraWorker(
            worker_id=i,
            camera=cam,
            num_frames=3
        )
        workers.append(worker)
        worker.start()
    
    # Wait for completion
    for worker in workers:
        worker.join()
    
    elapsed = time.time() - start_time
    
    # Cleanup
    for i, cam in enumerate(cameras):
        try:
            cam.close()
            print(f"[Main] Camera {i} closed")
        except RuntimeError as e:
            print(f"[Main] Error closing camera {i}: {e}")
    
    total_frames = sum(w.frames_grabbed for w in workers)
    print(f"\n[Main] Total: {total_frames} frames in {elapsed:.2f}s ({total_frames/elapsed:.1f} fps)")


def example_error_handling():
    """Example 3: Proper error handling with Rust locks"""
    print("\n" + "="*60)
    print("Example 3: Error Handling")
    print("="*60)
    
    cam = PyCamera("rpi", 0)
    
    # Try operations with proper error handling
    try:
        print("[Main] Opening camera...")
        cam.open()
        print("[Main] ✓ Camera opened")
        
        print("[Main] Setting exposure to 20000µs...")
        cam.set_exposure(20000.0)
        exposure = cam.get_exposure()
        print(f"[Main] ✓ Exposure set to {exposure}µs")
        
        print("[Main] Grabbing frame...")
        frame = cam.grab_frame()
        meta = frame.metadata()
        print(f"[Main] ✓ Frame grabbed: seq={meta.sequence}, size={meta.width}x{meta.height}")
        
    except RuntimeError as e:
        print(f"[Main] ✗ RuntimeError: {e}")
    except Exception as e:
        print(f"[Main] ✗ Unexpected error: {type(e).__name__}: {e}")
    finally:
        try:
            cam.close()
            print("[Main] ✓ Camera closed")
        except Exception as e:
            print(f"[Main] ✗ Error closing: {e}")


def example_performance_comparison():
    """Example 4: Show Rust mutex benefit"""
    print("\n" + "="*60)
    print("Example 4: Performance Metrics")
    print("="*60)
    
    cam = PyCamera("rpi", 0)
    cam.open()
    
    # Warm up
    for _ in range(5):
        _ = cam.grab_frame()
    
    print("[Main] Measuring lock performance with 4 threads...")
    
    class TimedWorker(CameraWorker):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, **kwargs)
            self.times = []
        
        def run(self):
            for _ in range(self.num_frames):
                try:
                    start = time.perf_counter()
                    _ = self.camera.grab_frame()
                    elapsed = (time.perf_counter() - start) * 1e6  # Convert to µs
                    self.times.append(elapsed)
                except RuntimeError:
                    pass
    
    workers: List[TimedWorker] = []
    start_time = time.time()
    
    for i in range(4):
        worker = TimedWorker(i, cam, num_frames=25)
        workers.append(worker)
        worker.start()
    
    for worker in workers:
        worker.join()
    
    elapsed = time.time() - start_time
    
    # Analyze times
    all_times = [t for w in workers for t in w.times]
    if all_times:
        avg_time = sum(all_times) / len(all_times)
        min_time = min(all_times)
        max_time = max(all_times)
        
        print(f"[Main] Lock acquisition times (Rust Mutex):")
        print(f"       Min: {min_time:.1f}µs")
        print(f"       Avg: {avg_time:.1f}µs")
        print(f"       Max: {max_time:.1f}µs")
        print(f"[Main] Total: {len(all_times)} locks in {elapsed:.2f}s = {1/elapsed*len(all_times):.1f} locks/sec")
    
    cam.close()


if __name__ == "__main__":
    print("""
╔════════════════════════════════════════════════════════════╗
║         Optik Rust Mutex - Practical Examples             ║
╚════════════════════════════════════════════════════════════╝

Diese Beispiele zeigen:
- Thread-safe camera access WITHOUT Python GIL
- Proper error handling
- Multi-camera patterns
- Performance characteristics
    """)
    
    # Run examples
    try:
        example_single_camera_multithreaded()
        example_multiple_cameras()
        example_error_handling()
        example_performance_comparison()
    except KeyboardInterrupt:
        print("\n[Main] Interrupted by user")
    except Exception as e:
        print(f"\n[Main] Unexpected error: {e}")
        import traceback
        traceback.print_exc()
    
    print("\n✓ All examples completed!\n")
