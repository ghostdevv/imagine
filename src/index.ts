import puppeteer, {
    type BrowserWorker,
    type Browser,
} from '@cloudflare/puppeteer';
import renderer from './renderer';

export interface Env {
    IMAGINE: R2Bucket;
    BROWSER: BrowserWorker;
}

function parseQuery(pathname: any) {
    const name = decodeURIComponent(pathname)
        .slice(0, -4)
        .replace(/_/g, ' ')
        .trim()
        .toLowerCase()
        .slice(0, 60);

    const key = `generated/${name.replace(/ /gm, '-')}.gif`;

    return { key, name };
}

export default {
    async fetch(
        request: Request,
        env: Env,
        ctx: ExecutionContext,
    ): Promise<Response> {
        const url = new URL(request.url);

        if (url.pathname == '/') {
            url.pathname = '/imagine.gif';
            return Response.redirect(url.toString(), 307);
        }

        if (!url.pathname.trim().endsWith('.gif')) {
            return new Response('Not Found', {
                status: 404,
            });
        }

        // Check cache api
        const cacheKey = new Request(url.toString(), request);
        const existingResponse = await caches.default.match(cacheKey);
        if (existingResponse) return existingResponse;

        // Response fn
        const gif = (file: BodyInit) => {
            const response = new Response(file, {
                headers: {
                    'Access-Control-Allow-Origin': '*',
                    'Content-Type': 'image/gif',
                    'Cache-Control': 's-maxage=604800',
                },
            });

            ctx.waitUntil(caches.default.put(cacheKey, response.clone()));

            return response;
        };

        if (url.pathname == '/base.gif') {
            const file = await env.IMAGINE.get('base.gif');
            return gif(file?.body!);
        }

        const { key, name } = parseQuery(url.pathname);
        const file = await env.IMAGINE.get(key);

        if (file) {
            return gif(file.body);
        }

        let browser: Browser;

        try {
            browser = await puppeteer.launch(env.BROWSER);
        } catch {
            return new Response(
                'No browsers available, try again in a sec lol',
                {
                    status: 500,
                },
            );
        }

        const [page] = await browser.pages();

        await page.setContent(renderer(name));
        await page.waitForSelector('#done');

        const data = await page.$eval('#done', (el) => el.innerText);

        await page.close();
        await browser.close();

        const bytes = JSON.parse(data);
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/gif' });

        await env.IMAGINE.put(key, blob.stream());

        return gif(blob);
    },
};
