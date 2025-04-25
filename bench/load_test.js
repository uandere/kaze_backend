import http from 'k6/http';
import { check, sleep } from 'k6';

// ---------- OPTIONS ----------
export const options = {
    scenarios: {
        ramp_generate: {
            executor: 'ramping-vus',            // classic stage-based executor :contentReference[oaicite:2]{index=2}
            startVUs: 0,
            stages: [
                { duration: '30s', target: 25 },
                { duration: '1m', target: 150 },
                { duration: '1m', target: 300 },
                { duration: '1m', target: 100 },
                { duration: '30s', target: 0 },
            ],
        },
    },
    thresholds: {                           // basic SLO: 95-th < 1 s :contentReference[oaicite:3]{index=3}
        http_req_duration: ['p(95)<1000'],
    },
};

// ---------- CONSTANTS ----------
const BASE = 'https://www.kazeapi.uk';
const AUTH = { Authorization: 'Bearer dummy_token' };   // adjust if you wire real JWT

// ---------- VU CODE ----------
export default function () {
    // Map iteration → landlordN (1-based, cycles every 1000)
    const n = Math.floor(Math.random() * 1000) + 1;
    const payload = JSON.stringify({
        tenant_id: `landlord${n}`,
        landlord_id: `landlord${n}`,
        housing_id: 'housing1',
        _uid: `landlord${n}`
    });

    const res = http.post(
        `${BASE}/agreement/generate`,
        payload,
        { headers: { ...AUTH, 'Content-Type': 'application/json' } },    // JSON body pattern :contentReference[oaicite:4]{index=4}
    );

    check(res, { '200 OK': r => r.status === 200 });
    sleep(1);    // helps distribute load a bit
}
