#!/usr/bin/env python3
"""
Create PNG icons for Amberol from simple geometric shapes
This script generates 16x16 PNG icons with transparent backgrounds
"""

from PIL import Image, ImageDraw
import os

def create_icon_directory():
    """Create the icons directory if it doesn't exist"""
    icon_dir = "src/assets/icons"
    os.makedirs(icon_dir, exist_ok=True)
    return icon_dir

def create_media_playback_start():
    """Create play button (triangle pointing right)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Triangle pointing right
    draw.polygon([(3, 2), (3, 14), (13, 8)], fill=(46, 52, 54, 255))
    return img

def create_media_playback_pause():
    """Create pause button (two vertical bars)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Two vertical bars
    draw.rectangle([(3, 2), (6, 14)], fill=(46, 52, 54, 255))
    draw.rectangle([(10, 2), (13, 14)], fill=(46, 52, 54, 255))
    return img

def create_media_skip_backward():
    """Create skip backward button (double triangle left)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # First triangle (left)
    draw.polygon([(2, 8), (8, 2), (8, 14)], fill=(46, 52, 54, 255))
    # Second triangle (right)  
    draw.polygon([(8, 8), (14, 2), (14, 14)], fill=(46, 52, 54, 255))
    return img

def create_media_skip_forward():
    """Create skip forward button (double triangle right)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # First triangle (left)
    draw.polygon([(2, 2), (2, 14), (8, 8)], fill=(46, 52, 54, 255))
    # Second triangle (right)
    draw.polygon([(8, 2), (8, 14), (14, 8)], fill=(46, 52, 54, 255))
    return img

def create_media_playlist_consecutive():
    """Create consecutive/linear playback (arrows pointing right)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Top arrow
    draw.polygon([(1, 3), (11, 3), (11, 1), (15, 4), (11, 7), (11, 5), (1, 5)], fill=(46, 52, 54, 255))
    # Bottom arrow  
    draw.polygon([(1, 11), (11, 11), (11, 9), (15, 12), (11, 15), (11, 13), (1, 13)], fill=(46, 52, 54, 255))
    return img

def create_media_playlist_repeat():
    """Create repeat all (circular arrows)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Draw curved arrow paths
    # Top curved line with arrow
    draw.arc([(2, 2), (14, 10)], 0, 180, fill=(46, 52, 54, 255), width=2)
    draw.polygon([(12, 3), (15, 6), (12, 7)], fill=(46, 52, 54, 255))
    # Bottom curved line with arrow
    draw.arc([(2, 6), (14, 14)], 180, 360, fill=(46, 52, 54, 255), width=2)
    draw.polygon([(4, 9), (1, 10), (4, 13)], fill=(46, 52, 54, 255))
    return img

def create_media_playlist_repeat_song():
    """Create repeat one (circular arrows with "1")"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Draw curved arrow paths (smaller)
    draw.arc([(1, 1), (12, 9)], 0, 180, fill=(46, 52, 54, 255), width=1)
    draw.polygon([(10, 2), (12, 4), (10, 5)], fill=(46, 52, 54, 255))
    draw.arc([(1, 7), (12, 15)], 180, 360, fill=(46, 52, 54, 255), width=1)
    draw.polygon([(3, 10), (1, 11), (3, 13)], fill=(46, 52, 54, 255))
    # Draw "1" in center
    draw.rectangle([(14, 5), (15, 11)], fill=(46, 52, 54, 255))
    draw.rectangle([(13, 10), (15, 11)], fill=(46, 52, 54, 255))
    return img

def create_media_playlist_shuffle():
    """Create shuffle (crossed arrows)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Diagonal lines crossing
    draw.line([(2, 3), (6, 7)], fill=(46, 52, 54, 255), width=2)
    draw.line([(10, 7), (14, 3)], fill=(46, 52, 54, 255), width=2)
    draw.line([(2, 13), (6, 9)], fill=(46, 52, 54, 255), width=2)
    draw.line([(10, 9), (14, 13)], fill=(46, 52, 54, 255), width=2)
    # Arrow heads
    draw.polygon([(13, 1), (15, 3), (13, 5)], fill=(46, 52, 54, 255))
    draw.polygon([(13, 11), (15, 13), (13, 15)], fill=(46, 52, 54, 255))
    return img

def create_view_queue():
    """Create view queue (horizontal lines)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Horizontal lines representing queue
    draw.rectangle([(2, 3), (14, 5)], fill=(46, 52, 54, 255))
    draw.rectangle([(2, 7), (14, 9)], fill=(46, 52, 54, 255))
    draw.rectangle([(2, 11), (14, 13)], fill=(46, 52, 54, 255))
    return img

def create_view_queue_rtl():
    """Create view queue RTL (horizontal lines, right-aligned)"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    # Horizontal lines (right-aligned)
    draw.rectangle([(2, 3), (14, 5)], fill=(46, 52, 54, 255))
    draw.rectangle([(4, 7), (14, 9)], fill=(46, 52, 54, 255))
    draw.rectangle([(2, 11), (14, 13)], fill=(46, 52, 54, 255))
    return img

def create_simple_icon(name, draw_func):
    """Create a simple symbolic icon"""
    img = Image.new('RGBA', (16, 16), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    if name == "app-remove":
        # X symbol
        draw.line([(4, 4), (12, 12)], fill=(46, 52, 54, 255), width=2)
        draw.line([(12, 4), (4, 12)], fill=(46, 52, 54, 255), width=2)
    elif name == "audio-only":
        # Speaker symbol
        draw.polygon([(2, 6), (5, 6), (8, 3), (8, 13), (5, 10), (2, 10)], fill=(46, 52, 54, 255))
        draw.arc([(9, 5), (13, 11)], 0, 180, fill=(46, 52, 54, 255), width=1)
    elif name == "edit-clear-all":
        # Trash can
        draw.rectangle([(5, 2), (11, 3)], fill=(46, 52, 54, 255))
        draw.rectangle([(4, 4), (12, 14)], fill=(46, 52, 54, 255))
        draw.rectangle([(6, 6), (6, 12)], fill=(255, 255, 255, 255))
        draw.rectangle([(10, 6), (10, 12)], fill=(255, 255, 255, 255))
    elif name == "edit-select-all":
        # Checkmark in box
        draw.rectangle([(2, 2), (14, 14)], outline=(46, 52, 54, 255), width=1)
        draw.line([(4, 8), (7, 11)], fill=(46, 52, 54, 255), width=2)
        draw.line([(7, 11), (12, 5)], fill=(46, 52, 54, 255), width=2)
    elif name == "folder-music":
        # Folder with note
        draw.polygon([(2, 4), (6, 4), (8, 6), (14, 6), (14, 13), (2, 13)], fill=(46, 52, 54, 255))
        draw.ellipse([(9, 8), (11, 10)], fill=(255, 255, 255, 255))
        draw.line([(11, 7), (11, 9)], fill=(255, 255, 255, 255), width=1)
    elif name == "go-previous":
        # Arrow pointing left
        draw.polygon([(10, 3), (6, 8), (10, 13), (8, 13), (4, 8), (8, 3)], fill=(46, 52, 54, 255))
    elif name == "selection-mode":
        # Multiple selection boxes
        draw.rectangle([(2, 2), (8, 8)], outline=(46, 52, 54, 255), width=1)
        draw.rectangle([(8, 8), (14, 14)], outline=(46, 52, 54, 255), width=1)
        draw.line([(4, 5), (6, 7)], fill=(46, 52, 54, 255), width=1)
        draw.line([(6, 7), (7, 4)], fill=(46, 52, 54, 255), width=1)
    
    return img

def main():
    """Generate all PNG icons"""
    print("🎨 Creating PNG icons...")
    
    icon_dir = create_icon_directory()
    
    # Define all icons to create
    icons = {
        'media-playback-start-symbolic.png': create_media_playback_start,
        'media-playback-pause-symbolic.png': create_media_playback_pause,
        'media-skip-backward-symbolic.png': create_media_skip_backward,
        'media-skip-forward-symbolic.png': create_media_skip_forward,
        'media-playlist-consecutive-symbolic.png': create_media_playlist_consecutive,
        'media-playlist-repeat-symbolic.png': create_media_playlist_repeat,
        'media-playlist-repeat-song-symbolic.png': create_media_playlist_repeat_song,
        'media-playlist-shuffle-symbolic.png': create_media_playlist_shuffle,
        'view-queue-symbolic.png': create_view_queue,
        'view-queue-rtl-symbolic.png': create_view_queue_rtl,
    }
    
    # Create simple icons
    simple_icons = [
        'app-remove', 'audio-only', 'edit-clear-all', 'edit-select-all',
        'folder-music', 'go-previous', 'selection-mode'
    ]
    
    for icon_name in simple_icons:
        filename = f"{icon_name}-symbolic.png"
        icons[filename] = lambda name=icon_name: create_simple_icon(name, None)
    
    # Generate all icons
    for filename, create_func in icons.items():
        filepath = os.path.join(icon_dir, filename)
        img = create_func()
        img.save(filepath, 'PNG')
        print(f"  ✅ Created {filename}")
    
    print(f"🎉 Generated {len(icons)} PNG icons in {icon_dir}")

if __name__ == "__main__":
    main()