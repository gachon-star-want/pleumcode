# pleumcode

PleumRouter 사용자를 위한 터미널 코딩 에이전트. [Goose](https://github.com/block/goose)(Apache-2.0)를
fork해서 만든다. 수익화 목적이 아니라, PleumRouter를 쓰는 사람들이 "우리 것"으로 코딩 에이전트를 한 번
써보게 하는 것이 목표. 계속 오픈소스로 유지한다.

## 상태

설계 문서 작성 완료, 코드 착수 전 (v0.1 준비 중). 진행 상황은 [`docs/00-vision.md`](docs/00-vision.md)에서
추적한다.

## 왜 Goose를 fork하는가

기존 오픈소스 코딩 에이전트 9종(opencode·OpenHands·Aider·Crush·Kilo Code·Hermes Agent·OpenClaw 등)을
조사해 비교한 결과다. 자세한 근거와 순위는 [`docs/04-fork-plan.md`](docs/04-fork-plan.md) 참고.

## 무엇이 다른가

다른 어떤 오픈소스 코딩 에이전트도 갖지 못한, PleumRouter 서버사이드 인프라 없이는 불가능한 기능들.
[`docs/01-unique-features.md`](docs/01-unique-features.md) 참고. 핵심은 **역할별로 다른 모델을 배정하는
멀티모델 서브에이전트 오케스트레이션** — Fusion/Cascade/Smart Mode는 전부 이 엔진 위의 프리셋이다.

## 문서

| 문서 | 내용 |
|---|---|
| [`docs/00-vision.md`](docs/00-vision.md) | 비전과 확정된 결정 사항 |
| [`docs/01-unique-features.md`](docs/01-unique-features.md) | PleumRouter만의 독창 기능 |
| [`docs/02-oss-survey.md`](docs/02-oss-survey.md) | 오픈소스 코딩 에이전트 45개 전수조사 + 기능 분류 |
| [`docs/03-harness-design.md`](docs/03-harness-design.md) | 코딩 하네스 설계 원칙 (신뢰성·벤치마크·보안) |
| [`docs/04-fork-plan.md`](docs/04-fork-plan.md) | Goose fork 근거와 착수 계획 |
| [`docs/05-roadmap.md`](docs/05-roadmap.md) | v0.1 ~ v1.0 로드맵 |

## License

Apache-2.0 (upstream [Goose](https://github.com/block/goose) 라이선스 상속).
