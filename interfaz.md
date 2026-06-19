<!-- Dashboard de Precision PDF -->
<!DOCTYPE html>

<html class="dark" lang="es"><head>
<meta charset="utf-8"/>
<meta content="width=device-width, initial-scale=1.0" name="viewport"/>
<title>Precision PDF - Dashboard</title>
<script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&amp;display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&amp;display=swap" rel="stylesheet"/>
<script id="tailwind-config">
      tailwind.config = {
        darkMode: "class",
        theme: {
          extend: {
            "colors": {
                    "surface-container": "#1e1f25",
                    "tertiary-container": "#6e2c00",
                    "on-error-container": "#ffdad6",
                    "background": "#121318",
                    "on-secondary": "#213145",
                    "on-primary-fixed": "#00164e",
                    "surface-variant": "#34343a",
                    "on-secondary-fixed-variant": "#38485d",
                    "on-tertiary-container": "#f39461",
                    "on-tertiary": "#552000",
                    "surface-dim": "#121318",
                    "on-secondary-fixed": "#0b1c30",
                    "primary-fixed-dim": "#b6c4ff",
                    "on-tertiary-fixed-variant": "#773205",
                    "error-container": "#93000a",
                    "inverse-primary": "#4059aa",
                    "outline": "#8f909d",
                    "on-surface-variant": "#c5c5d3",
                    "on-tertiary-fixed": "#341100",
                    "tertiary-fixed": "#ffdbcb",
                    "tertiary": "#ffb691",
                    "on-primary": "#05297a",
                    "on-primary-container": "#90a8ff",
                    "on-error": "#690005",
                    "primary-container": "#1e3a8a",
                    "on-secondary-container": "#a9bad3",
                    "on-primary-fixed-variant": "#264191",
                    "tertiary-fixed-dim": "#ffb691",
                    "outline-variant": "#444651",
                    "surface": "#121318",
                    "secondary-fixed-dim": "#b7c8e1",
                    "surface-container-high": "#292a2f",
                    "surface-container-lowest": "#0d0e13",
                    "surface-tint": "#b6c4ff",
                    "on-surface": "#e3e1e9",
                    "inverse-surface": "#e3e1e9",
                    "secondary-container": "#3a4a5f",
                    "inverse-on-surface": "#2f3036",
                    "on-background": "#e3e1e9",
                    "surface-container-highest": "#34343a",
                    "error": "#ffb4ab",
                    "secondary-fixed": "#d3e4fe",
                    "primary-fixed": "#dce1ff",
                    "secondary": "#b7c8e1",
                    "surface-container-low": "#1a1b21",
                    "primary": "#b6c4ff",
                    "surface-bright": "#38393f"
            },
            "borderRadius": {
                    "DEFAULT": "0.125rem",
                    "lg": "0.25rem",
                    "xl": "0.5rem",
                    "full": "0.75rem"
            },
            "spacing": {
                    "sidebar-width": "240px",
                    "gutter": "16px",
                    "stack-gap": "8px",
                    "margin-mobile": "16px",
                    "toolbar-width": "56px",
                    "margin-desktop": "24px"
            },
            "fontFamily": {
                    "headline-md": ["Inter"],
                    "label-md": ["Inter"],
                    "body-md": ["Inter"],
                    "body-lg": ["Inter"],
                    "headline-lg-mobile": ["Inter"],
                    "label-sm": ["Inter"],
                    "headline-lg": ["Inter"]
            },
            "fontSize": {
                    "headline-md": ["18px", {"lineHeight": "24px", "letterSpacing": "-0.01em", "fontWeight": "600"}],
                    "label-md": ["12px", {"lineHeight": "16px", "letterSpacing": "0.05em", "fontWeight": "500"}],
                    "body-md": ["14px", {"lineHeight": "20px", "fontWeight": "400"}],
                    "body-lg": ["16px", {"lineHeight": "24px", "fontWeight": "400"}],
                    "headline-lg-mobile": ["20px", {"lineHeight": "28px", "fontWeight": "600"}],
                    "label-sm": ["11px", {"lineHeight": "14px", "fontWeight": "500"}],
                    "headline-lg": ["24px", {"lineHeight": "32px", "letterSpacing": "-0.02em", "fontWeight": "600"}]
            }
          },
        },
      }
    </script>
<style>
        body { font-family: 'Inter', sans-serif; }
        .material-symbols-outlined {
            font-variation-settings: 'FILL' 0, 'wght' 300, 'GRAD' 0, 'opsz' 20;
        }
        .canvas-shadow {
            box-shadow: 0px 10px 30px rgba(0,0,0,0.15);
        }
        .shimmer {
            background: linear-gradient(90deg, transparent, rgba(182, 196, 255, 0.2), transparent);
            background-size: 200% 100%;
            animation: shimmer-anim 2s infinite linear;
        }
        @keyframes shimmer-anim {
            0% { background-position: -200% 0; }
            100% { background-position: 200% 0; }
        }
        ::-webkit-scrollbar { width: 6px; }
        ::-webkit-scrollbar-track { background: transparent; }
        ::-webkit-scrollbar-thumb { background: #444651; border-radius: 3px; }
        ::-webkit-scrollbar-thumb:hover { background: #8f909d; }
    </style>
</head>
<body class="bg-background text-on-surface overflow-hidden">
<!-- UI Content here -->
</body>
</html>
