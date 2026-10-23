import type { Metadata } from 'next';
import './globals.css';
export const metadata: Metadata = {
  title: 'Open Source Discovery Lab',
  description:
    'Local, currently non-commercial community research to inspire useful open-source software contributions.',
};
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
