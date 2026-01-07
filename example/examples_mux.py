#!/usr/bin/env python3
"""
Example: Using the Multiplexer Client
"""

from optik.mux import MuxClient
from optik.exceptions import OptikError


def main():
    """Connect to mux server and grab frames"""
    print("🌐 optik - Multiplexer Client Example")
    print("=" * 50)
    print("\nMake sure mux server is running:")
    print("  optik mux-server --port 5555\n")

    try:
        with MuxClient("127.0.0.1", 5555) as client:
            # List cameras
            print("📋 Available cameras:")
            cameras = client.list_cameras()
            for cam in cameras.get("cameras", []):
                print(f"  - {cam['index']}: {cam['serial']} ({cam['vendor']})")

            # Ping server
            print("\n🏓 Pinging server...")
            response = client.ping()
            print(f"  Status: {response.get('status')}")

            # Grab frame from first camera
            if cameras.get("cameras"):
                print("\n📸 Grabbing frame from camera 0...")
                frame = client.get_frame(0)
                print(
                    f"✅ Frame: {frame.shape[1]}x{frame.shape[0]}, "
                    f"Channels: {frame.shape[2] if len(frame.shape) > 2 else 1}"
                )

                # Adjust settings
                print("\n⚙️ Setting exposure to 20000 µs...")
                client.set_exposure(0, 20000)
                print("✅ Done")

    except OptikError as e:
        print(f"❌ Error: {e}")
    except Exception as e:
        print(f"❌ Unexpected error: {e}")


if __name__ == "__main__":
    main()
