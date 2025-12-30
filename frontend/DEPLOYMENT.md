# Deployment Guide

## Pre-deployment Checklist

- [ ] All dependencies installed (`npm install`)
- [ ] Build passes without errors (`npm run build`)
- [ ] Type checking passes (`npm run type-check`)
- [ ] Linting passes (`npm run lint`)
- [ ] GraphQL endpoint is accessible
- [ ] Environment variables configured

## Production Build

```bash
npm run build
```

This creates an optimized production build in the `dist/` directory with:
- Minified JavaScript and CSS
- Code splitting by route
- Source maps for debugging
- Optimized images and assets

## Environment Variables

Set these in your hosting platform:

```
VITE_GRAPHQL_ENDPOINT=https://your-api-domain.com/graphql
VITE_POLL_INTERVAL=2000
```

## Deployment Options

### Option 1: Vercel (Recommended)

1. Install Vercel CLI:
   ```bash
   npm install -g vercel
   ```

2. Deploy:
   ```bash
   vercel
   ```

3. Set environment variables in Vercel dashboard

4. Domain will be: `your-project.vercel.app`

### Option 2: Netlify

1. Build the project:
   ```bash
   npm run build
   ```

2. Deploy via Netlify CLI:
   ```bash
   npm install -g netlify-cli
   netlify deploy --prod --dir=dist
   ```

3. Or drag and drop `dist/` folder to Netlify dashboard

### Option 3: Cloudflare Pages

1. Connect your GitHub repository
2. Configure build settings:
   - Build command: `npm run build`
   - Build output: `dist`
3. Set environment variables
4. Deploy

### Option 4: AWS S3 + CloudFront

1. Build the project:
   ```bash
   npm run build
   ```

2. Create S3 bucket with static website hosting

3. Upload `dist/` contents to S3:
   ```bash
   aws s3 sync dist/ s3://your-bucket-name --delete
   ```

4. Create CloudFront distribution pointing to S3

5. Configure custom domain and SSL

### Option 5: Docker

Use this Dockerfile:

```dockerfile
# Build stage
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

Create `nginx.conf`:

```nginx
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
}
```

Build and run:

```bash
docker build -t fair-launch-frontend .
docker run -p 80:80 fair-launch-frontend
```

## Performance Optimization

### 1. Enable Compression

Most hosting platforms enable this by default. For custom servers:

```nginx
gzip on;
gzip_vary on;
gzip_min_length 1024;
gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;
```

### 2. CDN Configuration

Use a CDN (CloudFlare, AWS CloudFront) to:
- Cache static assets globally
- Reduce latency
- Handle traffic spikes

### 3. HTTP/2 and HTTP/3

Enable on your server for:
- Multiplexing
- Header compression
- Better performance

### 4. Preconnect to GraphQL API

Already configured in the app, but verify DNS prefetch:

```html
<link rel="preconnect" href="https://your-api-domain.com">
```

## Monitoring

### 1. Error Tracking

Add Sentry or similar:

```bash
npm install @sentry/react
```

Configure in `src/main.tsx`:

```typescript
import * as Sentry from "@sentry/react";

Sentry.init({
  dsn: "YOUR_SENTRY_DSN",
  environment: import.meta.env.MODE,
});
```

### 2. Analytics

Add Google Analytics, Plausible, or similar:

```html
<!-- Add to index.html -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
```

### 3. Performance Monitoring

Use Lighthouse CI or WebPageTest to track:
- First Contentful Paint (FCP)
- Largest Contentful Paint (LCP)
- Time to Interactive (TTI)
- Cumulative Layout Shift (CLS)

## Security Checklist

- [ ] HTTPS enabled (SSL certificate)
- [ ] Security headers configured
- [ ] No secrets in client-side code
- [ ] CORS properly configured on API
- [ ] Content Security Policy (CSP) headers
- [ ] Rate limiting on API endpoints
- [ ] Input validation and sanitization

## Post-Deployment

1. **Test all functionality**:
   - Browse tokens
   - Create token
   - Trade tokens
   - View portfolio
   - Wallet connection

2. **Performance audit**:
   ```bash
   npm install -g lighthouse
   lighthouse https://your-domain.com
   ```

3. **Load testing** (optional):
   ```bash
   npm install -g artillery
   artillery quick --count 10 -n 20 https://your-domain.com
   ```

4. **Monitor errors** in production

5. **Set up uptime monitoring** (UptimeRobot, Pingdom, etc.)

## Rollback Procedure

If deployment fails:

1. Revert to previous version in hosting dashboard
2. Or redeploy from previous git commit:
   ```bash
   git checkout <previous-commit>
   npm run build
   # Deploy again
   ```

## Continuous Deployment

Set up GitHub Actions for automatic deployment:

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
        with:
          node-version: '18'
      - run: npm ci
      - run: npm run build
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./dist
```

## Troubleshooting

### Blank Page After Deploy

- Check browser console for errors
- Verify base URL in `vite.config.ts`
- Check 404 errors for assets

### API Connection Fails

- Verify VITE_GRAPHQL_ENDPOINT
- Check CORS headers on API
- Verify API is accessible from production

### Build Fails

- Run `npm run type-check` locally
- Check Node.js version matches
- Clear node_modules and reinstall

## Support

For deployment issues:
1. Check hosting platform documentation
2. Review build logs
3. Test production build locally: `npm run preview`
4. Verify all environment variables are set
