# Asterinas ATC'25 Artifact Evaluation

This repository provides the artifacts and instructions to conduct artifact evaluation for the following paper.

> Yuke Peng, Hongliang Tian, Zhang Junyang, Ruihan Li, Chengjun Chen, Jianfeng Jiang, Jinyi Xian, Yingwei Luo, Xiaolin Wang, Chenren Xu, Diyu Zhou, Shoumeng Yan, Yinqian Zhang. _Asterinas: A Linux ABI-Compatible, Rust-Based Framekernel OS with a Small and Sound TCB_. USENIX ATC'25.

The main artifacts described by the paper can be found:
* OSTD: the [ostd/](ostd/) directory;
* Asterinas: the [kernel/](kernel/) directory;
* KernMiri: the [`kern_miri`](https://github.com/asterinas/atc25-artifact-evaluation/tree/kern_miri) branch. Note that KernMiri is a prototype and only supported by a [specific branch of OSTD](https://github.com/asterinas/atc25-artifact-evaluation/tree/miri_asterinas).

Follow [this evaluation document](eval/README.md) to reproduce the main results reported by the paper.

The most up-to-date version of Asterinas can be found [here](https://github.com/asterinas/asterinas).

-----

<p align="center">
    <img src="docs/src/images/logo_en.svg" alt="asterinas-logo" width="620"><br>
    A secure, fast, and general-purpose OS kernel written in Rust and compatible with Linux<br/>
</p>

## Introducing Asterinas

Asterinas is a _secure_, _fast_, and _general-purpose_ OS kernel
that provides _Linux-compatible_ ABI.
It can serve as a seamless replacement for Linux
while enhancing _memory safety_ and _developer friendliness_.

* Asterinas prioritizes memory safety
by employing Rust as its sole programming language
and limiting the use of _unsafe Rust_
to a clearly defined and minimal Trusted Computing Base (TCB).
This innovative approach,
known as [the framekernel architecture](https://asterinas.github.io/book/kernel/the-framekernel-architecture.html),
establishes Asterinas as a more secure and dependable kernel option.

* Asterinas surpasses Linux in terms of developer friendliness.
It empowers kernel developers to
(1) utilize the more productive Rust programming language,
(2) leverage a purpose-built toolkit called [OSDK](https://asterinas.github.io/book/osdk/guide/index.html) to streamline their workflows,
and (3) choose between releasing their kernel modules as open source
or keeping them proprietary,
thanks to the flexibility offered by [MPL](#License).

While the journey towards a production-grade OS kernel is challenging,
we are steadfastly progressing towards this goal.
Over the course of 2024,
we significantly enhanced Asterinas's maturity,
as detailed in [our end-year report](https://asterinas.github.io/2025/01/20/asterinas-in-2024.html).
In 2025, our primary goal is to make Asterinas production-ready on x86-64 virtual machines
and attract real users!

## Getting Started

Get yourself an x86-64 Linux machine with Docker installed.
Follow the three simple steps below to get Asterinas up and running.

1. Download the latest source code.

```bash
git clone --single-branch -b main https://github.com/asterinas/atc25-artifact-evaluation
```

2. Run a Docker container as the development environment.

```bash
docker run -it --privileged --network=host --device=/dev/kvm -v $(pwd)/atc25-artifact-evaluation:/root/asterinas asterinas/asterinas:0.14.1-20250326
```

3. Inside the container, go to the project folder to build and run Asterinas.

```bash
make build
make run
```

If everything goes well, Asterinas is now up and running inside a VM.

## The Book

See [The Asterinas Book](https://asterinas.github.io/book/) to learn more about the project.

## License

Asterinas's source code and documentation primarily use the 
[Mozilla Public License (MPL), Version 2.0](https://github.com/asterinas/asterinas/blob/main/LICENSE-MPL).
Select components are under more permissive licenses,
detailed [here](https://github.com/asterinas/asterinas/blob/main/.licenserc.yaml). For the rationales behind the choice of MPL, see [here](https://asterinas.github.io/book/index.html#licensing).
