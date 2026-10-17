# Generate your own background and HUD

Copy the prompt below into your preferred image generator. Use an original character or artwork you have permission to use. AI generation alone does not establish rights to recognised franchise characters.

## Background prompt
Create an original anime-inspired illustration for a tiny 320×240 (4:3 landscape) PC display. An original adult orbital engineer with a teal bob haircut and amber eyes occupies the right third. Midnight sky and cyan light, crisp expressive linework, clear silhouette. Leave the left half and lower third dark and visually quiet for sensor labels. No text, numbers, logo, gauges, borders or interface. No existing characters, costume replicas or artist imitation.

Export/crop to exactly 320×240, below 5 MB, and save as `themes/my-theme/art.png`. Do not put real readings into the background; the app renders them dynamically. Review the result at actual size, not only zoomed in.

## HUD planning prompt
I have a 320×240 background with the character on the right. Design a readable HUD using PRC Display Studio. Keep rectangles inside the canvas, reserve the face region and use no more than 64 elements. Suggest a CPU temperature number, GPU temperature number and RAM bar. Use large high-contrast values and short labels. Produce a layout plan with x/y/width/height and colours, then use the real `themes/example/theme.json` and Rust Element schema as the authoritative format. Do not invent sensor IDs or keys. Unknown readings must remain unavailable, never invented.

Use the editor for final validation. Background generation and HUD composition are separate steps. Treat AI JSON as a draft; test with sample and missing sensor data. Do not run AI-provided shell commands or download executable helpers to import a picture.
