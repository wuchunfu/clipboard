import type { ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";
import Heading from "@theme/Heading";
import Translate, { translate } from "@docusaurus/Translate";

import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">
          <Translate id="homepage.tagline">
            A modern, lightweight, and secure clipboard manager
          </Translate>
        </p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/installation"
          >
            <Translate id="homepage.download">Download Now 🚀</Translate>
          </Link>
          <span style={{ margin: "0 10px" }}></span>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro"
          >
            <Translate id="homepage.getStarted">Get Started ⏱️</Translate>
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title}`}
      description="A modern, lightweight, and secure clipboard manager for macOS, Windows, and Linux."
    >
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
