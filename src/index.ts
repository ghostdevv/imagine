import puppeteer, { type BrowserWorker } from '@cloudflare/puppeteer';
import renderer from './renderer';

export interface Env {
    IMAGINE: R2Bucket;
    BROWSER: BrowserWorker;
}

function parseQuery(pathname: any) {
    const name = decodeURIComponent(pathname)
        .slice(0, -4)
        .trim()
        .toLowerCase()
        .replace(/[^a-zA-Z0-9!? ]/gm, '');

    const key = `generated/${name.replace(/ /gm, '-')}.gif`;

    return { key, name };
}

function gif(file: BodyInit) {
    return new Response(file, {
        headers: {
            'Access-Control-Allow-Origin': '*',
            'Content-Type': 'image/gif',
        },
    });
}

export default {
    async fetch(
        request: Request,
        env: Env,
        ctx: ExecutionContext,
    ): Promise<Response> {
        const url = new URL(request.url);

        if (url.pathname == '/base.gif') {
            const file = await env.IMAGINE.get('base.gif');
            return gif(file?.body!);
        }

        if (url.pathname == '/') {
            url.pathname = '/imagine.gif';
            return Response.redirect(url.toString(), 307);
        }

        if (!url.pathname.trim().endsWith('.gif')) {
            return new Response('Not Found', {
                status: 404,
            });
        }

        const { key, name } = parseQuery(url.pathname);
        const file = await env.IMAGINE.get(key);

        if (file) {
            return gif(file.body);
        }

        const browser = await puppeteer.launch(env.BROWSER);

        const [page] = await browser.pages();

        await page.setContent(renderer(name));
        await page.waitForSelector('#done');

        const data = await page.$eval('#done', (el) => el.innerText);
        const bytes = JSON.parse(data);

        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/gif' });

        await env.IMAGINE.put(key, blob.stream());

        return gif(blob);
    },
};
