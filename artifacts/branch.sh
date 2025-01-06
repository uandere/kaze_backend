curl -X POST "https://api2s.diia.gov.ua/api/v2/acquirers/branch" \
-H "accept: application/json" \
-H "Content-Type: application/json" \
-H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJkYXRhIjoiZXlKbGJtTWlPaUpCTVRJNFEwSkRMVWhUTWpVMklpd2lZV3huSWpvaVVsTkJMVTlCUlZBaUxDSnJhV1FpT2lJME5UbHpTVzlaYmpFMVltVXRkRGxaYldKR01HcDFTbkl0ZDI5b2EyUTRSUzFKY0VWcVJrMXFURlUwSW4wLnJJU3VUU1QwTmVPZFpybVhUWmpwWU0wZXBQYlhrRWdpd0FRcDc1RFFWQXdfbS1OUDY4REdoVWxwUVc4WnlTTnNNWU9PNUwweW9sLUUxMmc0ZlRvajJQNG9vbS1HcHJtU3NKUWZFMWw4RXp1YURHTFRUOEktUF9TblhjVnpGUFJ3bE5jMWJyY2pWTnJoQlphWW54cU1lQU1icTFtRWhqNnRTZ0FzNTNyOWxUbjFjVUhGZDJfbGE4aEVzX0xybHZMaWRlSVlHMVlTRnBrV29taGpMcmdMY0VaM3EtNmFlV0hvZExpTWRfTmxxTmp2a2xCN1NHUzVGOWRnODREdHYyY1BUWWk2ZW5ZXzdUYnpfSEdRTnAwUkhRNThacmphUnQ0STh4OF9aVGxjWGFrZkJVcWRVMEJCWkptRVNQMERhcHBZNy1xZk1xRUlsdk01ckhwQ1Brd1hLdy5sb3NFZXRuSGlZTHc1SXZDR2g1YUR3Lk1ySjVPTUJaZjdKWW9aN3l1YVZORjgxbXcwOU1ZZXh1M0Zfc2wxUUFHaGRZSDYwZFU3TDQ4cHlZXzBKYW1KTGVRb0FFOXBSUk5Wb1g0OTFSTi1wd2ZyYTI3ZEVuWmhVRjExU241dVVKMG1FWGMzcnRZSW9mT1lCRm01UW1qTTZVbERsLV9meDdaNE9FcTBCR3huSmVid09venhQWjZsTV85dUIzamtUbEwyZmNSMEdnMGpWZl9sdEJEdkJKNFlSeTgzUE5YRVpVX1pzejlRNHNERUtqY1EuWnoxbElleHRXcFJ2dWEwbE5CQ0Q4QSIsImlhdCI6MTczNjE3NTE3MiwiZXhwIjoxNzM2MTgyMzcyfQ.O1zs1W8lujM-EbXNbNb4rP3XMirmF_Q1ajbwItL0tj7sDlWRC9yy_cZ8at1O8arbnmKADybNXn2kC_GIlB6y1xKF_oQ-bAhG2arHCBRTB5VimY3hfkW_9Q9rF3Se1OZCq0Jaxd2i-VO_eniiibgz-r8DPU0hWoVAnIYFxVJjIPPWxkBbAmYMp6XpO5JiE5XN0wZfBJ89-SipIyE8tEYhdC8AHgUSoo1Ju27EmlygzvdRtz2rIW6G2KQZEpWYhK77zf8UZtKF5GPVbydD7rQk9U1UGqik02xa5IGdrdba5HZ2D6_kYBtL-6JGR1aOVFt53hBd-_37j4zesMvP5cVSkQ" \
-d '{
        "name": "Kaze Ukraine",
        "email": "kazerealty.inc@gmail.com",
        "region": "Львівська",
        "district": "Львівський",
        "location": "Львів",
        "street": "Пимоненка",
        "house": "7к",
        "deliveryTypes": ["api"],
        "offerRequestType": "dynamic",
        "scopes": {
            "sharing": [
                "passport", "internal-passport", "foreign-passport", "taxpayer-card", "student-id-card", "reference-internally-displaced-person"
            ],
            "diiaId": [
                "hashedFilesSigning"
            ]
        }
}'
