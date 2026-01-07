#!/usr/bin/env python3
"""
Example: Basic RPi camera usage
"""

from optik import MultiController


def main():
    """Discover and print camera information"""
    print("🎥 optik - RPi Camera Discovery Example")
    print("=" * 50)

    with MultiController() as ctrl:
        cameras = ctrl.discover()

        if not cameras:
            print("❌ No cameras found")
            print("Make sure:")
            print("  1. Camera is enabled: vcgencmd get_camera")
            print("  2. Camera is connected")
            print("  3. User is in 'video' group: groups $USER")
            return

        print(f"✅ Found {len(cameras)} cameras:\n")

        for i, camera in enumerate(cameras):
            print(f"  Camera {i}:")
            print(f"    Serial: {camera.serial}")
            print(f"    Vendor: {camera.vendor}")
            print(f"    Exposure: {camera.get_exposure():.2f} µs")
            print(f"    Gain: {camera.get_gain():.2f} dB")
            print(f"    Format: {camera.get_pixel_format()}")
            print()

        # Try to grab frame from first camera
        if cameras:
            print("📸 Grabbing frame from first camera...")
            frame = cameras[0].safe_get_image()
            if frame is not None:
                print(
                    f"✅ Frame: {frame.shape[1]}x{frame.shape[0]}, "
                    f"Channels: {frame.shape[2] if len(frame.shape) > 2 else 1}"
                )
            else:
                print("❌ Failed to grab frame")


if __name__ == "__main__":
    main()
