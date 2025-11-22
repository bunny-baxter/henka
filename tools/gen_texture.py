#!/usr/bin/env python3

# Script by Claude

"""
Generate a white noise image.

This script creates an image filled with random pixel values (white noise)
and saves it to the specified output file.
"""

import argparse
import numpy as np
from PIL import Image

# File constants
IMAGE_WIDTH = 128
IMAGE_HEIGHT = 128
IMAGE_MODE = 'RGB'


def generate_white_noise(width, height):
    """
    Generate a white noise image.
    
    Args:
        width: Width of the image in pixels
        height: Height of the image in pixels
    
    Returns:
        PIL Image object containing white noise
    """
    rng = np.random.default_rng()

    # Generate random values between 0 and 255 for each pixel
    base_noise = rng.integers(low=0, high=256, size=(height, width))
    
    # Stack the same values across all three RGB channels
    img_array = np.stack([base_noise, base_noise, base_noise], axis=2)

    # Add some more noise to each channel
    CHANNEL_NOISE = 24
    channel_noise = rng.integers(low=-CHANNEL_NOISE, high=CHANNEL_NOISE, size=(height, width, 3))
    img_array = np.clip(img_array + channel_noise, 0, 255).astype(np.uint8)
    
    return Image.fromarray(img_array, mode=IMAGE_MODE)


def main():
    parser = argparse.ArgumentParser(
        description='Generate a white noise image'
    )
    parser.add_argument(
        'output',
        help='Output filename for the generated image'
    )
    
    args = parser.parse_args()
    
    # Generate the white noise image
    print(f"Generating {IMAGE_WIDTH}x{IMAGE_HEIGHT} white noise image...")
    image = generate_white_noise(IMAGE_WIDTH, IMAGE_HEIGHT)
    
    # Save the image as PNG
    image.save(args.output, format='PNG')
    print(f"Image saved to: {args.output}")


if __name__ == '__main__':
    main()
