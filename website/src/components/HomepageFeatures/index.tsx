import type { ReactNode } from "react";
import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";
import { ShieldCheck, Zap, Monitor } from "lucide-react";
import Translate from "@docusaurus/Translate";

type FeatureItem = {
  title: ReactNode;
  Icon: React.ElementType;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: <Translate id="feature.secure.title">Secure & Private</Translate>,
    Icon: ShieldCheck,
    description: (
      <Translate id="feature.secure.description">
        Your data stays on your device. We use AES-GCM encryption to store your
        clipboard history locally, ensuring your sensitive information remains
        private and secure.
      </Translate>
    ),
  },
  {
    title: <Translate id="feature.fast.title">Lightning Fast</Translate>,
    Icon: Zap,
    description: (
      <Translate id="feature.fast.description">
        Built with Rust and Tauri, the application is incredibly lightweight and
        performant. Instant search and smooth scrolling, even with thousands of
        history items.
      </Translate>
    ),
  },
  {
    title: (
      <Translate id="feature.crossPlatform.title">Cross Platform</Translate>
    ),
    Icon: Monitor,
    description: (
      <Translate id="feature.crossPlatform.description">
        Whether you use macOS, Windows, or Linux, we've got you covered. Enjoy a
        consistent and native-like experience across all your desktop devices.
      </Translate>
    ),
  },
];

function Feature({ title, Icon, description }: FeatureItem) {
  return (
    <div className={clsx("col col--4")}>
      <div className="text--center">
        <Icon className={styles.featureSvg} size={64} />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
