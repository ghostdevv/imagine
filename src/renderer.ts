export default (text: string) => `<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta http-equiv="X-UA-Compatible" content="IE=edge" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link rel="preconnect" href="https://fonts.googleapis.com">
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
        <link href="https://fonts.googleapis.com/css2?family=Lilita+One&display=swap" rel="stylesheet">
    </head>
    <body>
        <img style="border: 4px solid green" id="baseGif" />
        <img style="border: 4px solid purple" id="output" />

        <br />
        <h1>Frames:</h1>

        <script type="module">
            // CONFIG

            const BASE_IMAGE = 'https://imagine.willow.sh/base.gif';
            const TEXT = String.raw\`${text}\`.toUpperCase();

            // SCRIPT

            import {
                GIFEncoder,
                quantize,
                applyPalette,
            } from 'https://esm.sh/gifenc@1.0.3';
            import getFrames from 'https://esm.sh/gif-frames@1.0.1';

            /** @type {HTMLImageElement} */
            const baseGif = document.querySelector('#baseGif');
            baseGif.src = BASE_IMAGE;

            /** @type {HTMLImageElement} */
            const output = document.querySelector('#output');

            const frames = await getFrames({
                outputType: 'canvas',
                url: BASE_IMAGE,
                frames: 'all',
            });

            const { width, height } = baseGif;

            const gif = new GIFEncoder();

            for (let i = 0; i < frames.length; i++) {
                const frame = frames[i];

                /** @type {HTMLCanvasElement} */
                const canvas = frame.getImage();
                const ctx = canvas.getContext('2d');

                if (i >= 8) {
                    ctx.fillStyle = \`rgba(255, 255, 255, \${(i - 8) * 0.1})\`;
                    ctx.font = "26px 'Lilita One'";
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.lineWidth = 1.5

                    const textX = width / 2;
                    const textY = height / 4.5;

                    ctx.fillText(TEXT, textX, textY);
                    ctx.strokeText(TEXT, textX, textY);
                }

                const { data } = ctx.getImageData(0, 0, width, height);

                const palette = quantize(data, 256);
                const index = applyPalette(data, palette);

                gif.writeFrame(index, width, height, { palette });
                document.body.appendChild(canvas);
            }

            gif.finish();
            const bytes = gif.bytes();

            output.src = URL.createObjectURL(new Blob([bytes], { type: 'text/gif' }))

            const doneElement = document.createElement('p');

            doneElement.id = 'done';
            doneElement.style.display = 'none';
            doneElement.innerText = JSON.stringify([...gif.bytes()]);

            document.body.appendChild(doneElement);
        </script>
    </body>
</html>
`;
