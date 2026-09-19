# pleumcode — 비전 & 진행 로그

## 목적

PleumRouter를 쓰는 사람들이 "우리 것"으로 코딩 에이전트를 한 번 써보게 만드는 것. 수익화가 목적이 아니다.
오픈소스로 계속 유지하고 GitHub에 Public으로 공개한다.

단순히 기존 오픈소스 코딩 에이전트에 로고만 바꿔 다는 건 안 한다. 이유가 없으면 안 만든다(YAGNI) — 만들 거면:

1. 다른 오픈소스 코딩 에이전트들이 이미 검증한 핵심 기능들을 최대한 흡수한다(전수조사 30개 이상).
2. PleumRouter가 아니면 못 만드는 독창적 기능을 최소 1개 이상 갖춘다.
3. 코딩 자체를 체계적으로, 견고하게 잘하는 에이전트를 만든다 — 최신 논문/트렌드 기반 하네스 설계.

## 결정된 것

- **Base: Goose fork** (`block/goose`, Apache-2.0, Rust). 근거는 [`04-fork-plan.md`](./04-fork-plan.md).
- 기존 `pleumrouter` npm CLI(`pleum` 바이너리)는 건드리지 않는다 — 새 바이너리(가칭 `pleum-code`)로 별도 배포하고, 안정화 후 `pleum launch`의 신규 레시피로 편입할지 검토.
- 서버사이드 로직(Smart Mode/Fusion/routing-policies 판단)은 절대 fork 쪽에 재구현하지 않는다 — 항상 모델 ID 문자열 패스스루로만 유지(`pleum/web/routers/routing_policies.py`·`fusion.py`가 유일한 진실 소스).

## 진행 중 (인터뷰 + 리서치 병행)

- 인터뷰: 독창적 기능 우선순위, 저장소/조직 이름, v1 범위 — 사용자와 협의 중.
- 리서치 워크플로우: 오픈소스 코딩 에이전트 30개+ 전수조사 + 최신 하네스/벤치마크 트렌드 — 백그라운드 실행 중, 결과가 [`02-oss-survey.md`](./02-oss-survey.md)·[`03-harness-design.md`](./03-harness-design.md)에 반영될 예정.

## 확정된 것 (2026-09-19 인터뷰)

- **v1 배포 범위**: 터미널 CLI만 (Goose 그대로의 형태). IDE 확장은 로드맵으로만 문서화, 지금은 범위 밖.
- **저장소**: 이름 `pleumcode`, PleumRouter와 같은 GitHub 조직에 Public으로 공개.
- **v1 독창적 기능**: [`01-unique-features.md`](./01-unique-features.md) 확정 — 핵심은 "역할별로 다른
  모델을 배정하는 멀티모델 서브에이전트 오케스트레이션"이며, Fusion/Cascade/Smart Mode는 이 엔진 위의
  프리셋으로 통합 설계한다. + 멀티모달 tool 통합, KRW 비용/예산 가드레일.

## 남은 것

- [x] 오픈소스 코딩 에이전트 45개 전수조사 완료([`02-oss-survey.md`](./02-oss-survey.md), 목표 30개 초과 달성)
- [x] 하네스 설계 원칙 확정([`03-harness-design.md`](./03-harness-design.md), 11개 원칙 + 벤치마크 + 안전성 권고)
- [x] 로드맵 v0.1~v1.0 구체화([`05-roadmap.md`](./05-roadmap.md))
- [ ] **중요 발견**: Goose는 자체 OS 샌드박스가 없음 — 유료 KRW 크레딧으로 임의 명령을 실행하는 제품이라
      이건 v1 출시 전 반드시 메워야 할 최우선 아키텍처 갭. [`03-harness-design.md`](./03-harness-design.md)
      원칙 11, [`04-fork-plan.md`](./04-fork-plan.md) 참고.
- [ ] GitHub Public 저장소 생성 (문서 초안 갖춰졌으니 이제 진행 가능 — fork 실행 착수와 함께)

## v0.1 진행 (2026-09-19)

상세는 [`04-fork-plan.md`](./04-fork-plan.md) "진행 로그".

- [x] 라이선스/조직 재확인 — 정본은 `aaif-goose/goose`, `v1.51.0`에 고정, fork 병합 완료(브랜치 `feat/v0.1-bringup`, 미푸시)
- [x] provider json 배선 — core 무수정으로 로드·게이트웨이 도달 확인 (진짜 키로 200 확인은 남음)
- [x] goose → pleum 전면 리네임 — `pleum/rebrand.py`, 상세·매핑·병합 절차는 04 "전면 리네임" (바이너리 이름 충돌·시각 흔적은 미결)
- [x] 바이너리 `pleumcode` / `ui/`·`documentation/` 삭제 / plum 워드마크 배너 — 04 "후속 결정"
- [~] OS 커널 샌드박스 — macOS 첫 컷 완료(셸+write/edit). Linux/Windows·hooks·MCP spawn은 미완, 그 OS에선 fail-closed
- [~] 채택순서 1~6 — 1번(메모리 파일)은 **AGENTS.md 최우선 + CLAUDE.md fallback**으로 완료(`PLEUM.md`는 만들지 않음). 남은 것: 권한 정책 파일, shadow-git, 린트 게이트, staged-diff, 세션 재개
