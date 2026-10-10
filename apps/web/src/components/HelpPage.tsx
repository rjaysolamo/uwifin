'use client';
import Link from 'next/link';
import { ArrowUpRight, ChevronDown, CircleHelp, Heart, ShieldCheck, Zap } from 'lucide-react';
const faqs = [
  {
    id: 'wallet',
    question: 'Is this real money?',
    answer:
      'Balances are read from your connected smart wallet. Base mainnet uses real assets; Base Sepolia is a test network with tokens that have no monetary value. Check the network shown in your wallet.',
  },
  {
    id: 'network-fees',
    question: 'How do sponsored network fees work?',
    answer:
      'Eligible transfers can have their network fee paid through an Alchemy Gas Manager policy. Eligibility depends on the network, token, method, and spending limits. Transfers proceed only after sponsorship is approved.',
  },
  {
    id: 'network',
    question: 'Which assets and networks can I use?',
    answer:
      'This MVP focuses on USDC on a single Base network. Live development uses Base Sepolia testnet. The sender and receiver must use the same network. Never send another asset or use another chain without confirming support.',
  },
  {
    id: 'payouts',
    question: 'Can I send pesos to GCash or a Philippine bank?',
    answer:
      'PHP bank and GCash payouts are not available in this MVP. Those flows require an approved payout partner and the relevant operational permissions. The send flow currently represents wallet-to-wallet USDC transfers.',
  },
  {
    id: 'stripe',
    question: 'Can I buy crypto with my card?',
    answer:
      'Stripe crypto on-ramp is provider-dependent. It must support your country, currency, asset, and destination network, and the UwiFin platform must be eligible. When enabled, Stripe collects payment details directly. UwiFin displays the provider-verified payment status.',
  },
  {
    id: 'pending',
    question: 'Why is my transfer pending?',
    answer:
      'Pending means the transaction has been submitted and is waiting for confirmation. An HTTP success response alone does not prove settlement. Real transfers must be verified against the blockchain receipt.',
  },
  {
    id: 'security',
    question: 'Does UwiFin store my private key?',
    answer:
      'No. Your wallet provider manages your keys. UwiFin stores your account identity, wallet association, and transaction metadata. Never share your private key or seed phrase with anyone.',
  },
];
export function HelpPage() {
  return (
    <>
      <section className="page-heading">
        <div>
          <span className="eyebrow">A LITTLE HELP GOES A LONG WAY</span>
          <h1>
            We’re here to help<span className="greeting-dot">.</span>
          </h1>
          <p>A few clear answers, so you can move forward with confidence.</p>
        </div>
        <CircleHelp size={30} className="help-heading-icon" />
      </section>
      <div className="help-highlights">
        <div className="card">
          <ShieldCheck size={23} />
          <h3>Your security matters</h3>
          <p>Keep your keys private and always verify the recipient.</p>
        </div>
        <div className="card">
          <Zap size={23} />
          <h3>Less to think about</h3>
          <p>Sponsored fees for eligible transfers, with clear status tracking.</p>
        </div>
        <div className="card">
          <Heart size={23} />
          <h3>Made for your people</h3>
          <p>A simple experience for staying connected across borders.</p>
        </div>
      </div>
      <section className="card faq-card">
        <h2>A few things you might be wondering</h2>
        {faqs.map((item) => (
          <details id={item.id} key={item.id}>
            <summary>
              {item.question}
              <ChevronDown size={17} />
            </summary>
            <p>{item.answer}</p>
          </details>
        ))}
      </section>
      <div className="help-bottom">
        <span>Ready to explore?</span>
        <Link className="text-link" href="/">
          Back to your overview <ArrowUpRight size={15} />
        </Link>
      </div>
    </>
  );
}
