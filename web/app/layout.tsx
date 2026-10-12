import type { Metadata } from 'next';
import './globals.css';
export const metadata:Metadata={title:'Pain Intelligence OS',description:'Evidence-grounded discovery and validation of customer problems'};
export default function RootLayout({children}:{children:React.ReactNode}) {
 return <html lang="en"><body>{children}</body></html>;
}
