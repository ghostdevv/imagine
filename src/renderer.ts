export default (text: string) => `<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta http-equiv="X-UA-Compatible" content="IE=edge" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    </head>
    <body>
        <img style="border: 4px solid green" id="baseGif" />
        <img style="border: 4px solid purple" id="output" />

        <br />
        <h1>Frames:</h1>

        <script type="module">
            // CONFIG

            const BASE_IMAGE = 'https://imagine.willow.sh/base';
            const TEXT = String.raw\`${text}\`;

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

                if (i > 8) {
                    ctx.fillStyle = \`rgba(255, 255, 255, \${(i - 8) * 0.1})\`;
                    ctx.font = '24px Arial, sans-serif';
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.shadowColor = 'rgba(0, 0, 0, 0.5)';
                    ctx.shadowBlur = 4;

                    // Calculate the position to center the text
                    const textX = width / 2;
                    const textY = height / 4.5;

                    // Draw the text onto the canvas
                    ctx.fillText(TEXT, textX, textY);
                }

                const { data } = ctx.getImageData(0, 0, width, height);

                const palette = quantize(data, 256);
                const index = applyPalette(data, palette);

                gif.writeFrame(index, width, height, { palette });
                document.body.appendChild(canvas);
            }

            gif.finish();

            /** @param {Blob} blob */
            function getDataUri(blob) {
                return new Promise((resolve) => {
                    const reader = new FileReader();

                    reader.onload = () => {
                        resolve(reader.result);
                    };

                    reader.readAsDataURL(blob);
                });
            }

            /** @type {Uint8Array} */
            const bytes = gif.bytes();
            // const blob = new Blob([bytes], { type: 'image/gif' });

            // output.src = await getDataUri(blob);
            // console.log(output.src)

            // For puppeteer
            const done = document.createElement('p');
            done.id = 'done';
            done.innerText = JSON.stringify([...bytes]);
            document.body.appendChild(done);
        </script>
    </body>
</html>
`;
