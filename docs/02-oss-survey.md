# 오픈소스 코딩 에이전트 전수조사

> 리서치 완료 2026-09-19 (워크플로우 `wf_45fe42c2-dc8`, 10개 그룹 병렬 조사 → 1개 종합). 목표였던 "30개 이상"을
> 넘겨 **45개 프로젝트**를 실제 저장소/라이선스까지 검증했다. 원본 전체 데이터(프로젝트별 상세 설명·출처 URL)는
> `~/.claude/projects/.../tasks/watvg8t8z.output`에 보존되어 있음 — 이 문서는 실행 가능한 결론만 정리한 것.

## 어떻게 읽을까

아래 "카테고리별 기능 분류"가 핵심이다 — **기능을, 어느 프로젝트가 아니라 어떤 카테고리(컨텍스트/편집신뢰성/
안전성/...)에 속하는지로 정리**했고, 각 항목에 우선순위(`high`/`medium`/`low`)와 "왜 이 우선순위인가"를 달았다.
"채택 순서"는 이 분류를 실제 구현 시퀀스로 푼 것. 맨 아래 "프로젝트 색인"은 조사한 45개 프로젝트를 찾아보기용으로
정리한 표다.

**라이선스 주의**: `AutoCodeRover`(SONAR Source-Available), `GPT-Pilot`(FSL-1.1-MIT), `Sweep`(현재 대부분
Enterprise License), `CodeR`/`Lingma SWE-GPT`/`MASAI`(라이선스 미확인)는 **완전한 오픈소스가 아니거나 확인
불가** — 아이디어만 참고하고 코드는 절대 가져오지 않는다.

---

## 카테고리별 기능 분류

### 1. 컨텍스트 엔지니어링 & 레포 이해

- **[high]** Tree-sitter 기반 repo-map — 함수/클래스 시그니처 그래프를 PageRank류로 랭크해 토큰 예산 안에서 요약 *(Aider, Plandex)* — 코어 루프 무변경, tree-sitter crate가 Rust 네이티브라 Goose 이식 마찰 거의 없음. KRW 크레딧 관점에서 토큰비용을 가장 직접적으로 줄여줌.
- **[high]** AST 기반 구조 인지 코드 검색(search-by-class/function, grep 대체) *(AutoCodeRover, Moatless Tools, Refact)* — 새 tool 하나 추가로 끝남. AutoCodeRover는 라이선스 문제 있으니 tree-sitter query로 클린룸 재구현.
- **[medium]** 저장소 전체 호출/임포트 그래프를 플러그인형 컨텍스트로 주입 *(RepoGraph)* — repo-map/AST검색 안정화 후 2단계.
- **[high]** 계층적 git-버전관리 메모리 파일 컨벤션(AGENTS.md/GEMINI.md → **PLEUM.md**) *(Gemini CLI, Codex CLI, Qwen Code)* — 파일 하나 읽어 시스템 프롬프트에 병합하는 수준, 구현비용 거의 0. PleumRouter routing-policy를 레포별로 명시하는 자연스러운 훅.
- **[medium]** '@' 멘션 기반 pluggable 컨텍스트 프로바이더 레지스트리 *(Continue, Sourcegraph Cody)* — 크레딧 잔액/provider 상태/routing 히스토리를 core 프롬프트 안 고치고 주입하는 확장점.
- **[low]** 로컬 임베딩+BM25 하이브리드 인덱스 *(Continue)* — repo-map/AST검색으로 상당부분 대체 가능, 후순위.

### 2. 편집/패치 신뢰성 — Agent-Computer Interface

- **[high]** ACI: LM 전용 view/edit/search 커맨드 세트 *(SWE-agent)* — 원시 bash 대비 SWE-bench 점수 대폭 상승이 논문(NeurIPS 2024)으로 검증됨. 기존 edit tool 포맷만 다듬는 낮은 침습도.
- **[high]** 편집 적용 전 문법/린트 검사 게이트(파싱 실패 시 적용 거부) *(SWE-agent)* — wrapper 한 겹으로 끝나는 가장 값싼 신뢰성 개선.
- **[medium]** 모델별 최적 edit-format 자동 선택(search/replace, udiff, whole-file) *(Aider)* — 68 provider 다루는 PleumRouter라 모델별 편집 신뢰도 차이가 클 것.
- **[high]** Architect/Editor 2-모델 분리(추론모델 계획 + 별도 편집모델 diff적용), Fast Apply류 소형 고속 apply모델 포함 *(Aider, Avante.nvim)* — PleumRouter 멀티프로바이더 라우팅과 상품적으로 가장 잘 맞음(계획=비싼모델, 적용=저비용 고속모델). Smart Mode/Fusion으로 그대로 노출 가능.
- **[medium]** CodeAct: 도구호출을 JSON 대신 실행가능 코드 1회로 통일 *(CodeAct, OpenHands 채택)* — LLM 호출(=과금 이벤트) 수를 줄여 크레딧 원장과 궁합 좋으나, 액션 디스패치 재설계가 필요한 core급 변경이라 후순위.

### 3. 안전성, 샌드박싱 & 롤백

- **[high]** 매 tool 호출 후 사용자 실제 git과 분리된 shadow-git 자동 체크포인트 *(Cline, Void)* — 별도 git 레포 하나 추가로 끝나 core 변경 없음, 사용자 실제 히스토리 절대 안 건드려 사고 위험 최저.
- **[medium]** 매 편집마다 실레포에 자동 git commit + LLM 커밋메시지 *(Aider)* — shadow-git과 목적 겹침, 병행 여부는 설계 결정 필요.
- **[medium]** AI 변경을 실파일과 분리된 스테이징 영역에 누적, 승인 전 미적용 diff 리뷰 샌드박스 *(Plandex, bolt.diy)* — shadow-git보다 큰 서브시스템(플랜 단위 상태저장) 필요.
- **[high]** OS 커널 수준 샌드박스(macOS Seatbelt / Linux Landlock+seccomp)로 승인정책과 실행권한 분리 *(OpenAI Codex CLI)* — pleumcode가 이미 Rust라 codex-rs 샌드박스 코드가 가장 직접 이식 가능. **유료 크레딧으로 임의 명령 실행하는 만큼 필수.**
- **[medium]** Docker 샌드박스 실행 런타임(persistent bash/브라우저 소유, 구조화 Observation 반환) *(OpenHands)* — OS샌드박스보다 무거운 인프라, 원격/클라우드 실행 확장 시 도입.
- **[high]** 텔레메트리/원격실행 모듈은 반드시 샌드박스·서명검증 *(GPT-Pilot 공급망 공격 사례 — 반면교사)* — 코드 이식이 아니라 **설계 원칙**. 다른 모든 기능보다 먼저 확정해야 할 보안 전제조건.

### 4. 멀티모델 라우팅 & 비용 최적화 (PleumRouter 연계)

- **[high]** 역할별 모델 설정(chat/plan vs autocomplete vs editor/apply vs embed) 명시적 분리 *(Continue, Aider)* — 설정 스키마 확장만으로 가능, PleumRouter의 Smart Mode/Fusion이 라우팅 인프라를 이미 갖고 있으니 "어느 역할에 어느 provider"만 선언하면 됨.
- **[medium]** 에이전트 루프 없이 localization→repair→validation 고정 파이프라인으로 저비용·예측가능 모드 *(Agentless, Agentless-Lite)* — 자유루프 모드와 나란히 둘 "경량 모드" 추가 커맨드, 요금 예측성 필요 사용자에 바로 상품화.
- **[low]** 이슈당 비용(USD/토큰) 벤치마크 데이터를 라우팅 판단 근거로 공개 *(Moatless Tools)* — pleumcode 자체 벤치마크 쌓인 뒤 대시보드로.
- **[low]** 코딩 특화 자체/파인튜닝 SWE 모델 경로 *(Lingma SWE-GPT)* — 모델 학습 자체가 별도 프로젝트 규모, 장기 검토.

### 5. 에이전트 오케스트레이션 & 태스크 분해

- **[high]** 오케스트레이터가 복잡 작업을 컨텍스트 격리된 서브태스크로 위임, 요약만 회수(부모 컨텍스트창 보호) *(Roo Code "Boomerang Tasks")* — 컨텍스트 폭발을 막는 가장 직접적 해법. 기존 루프 위에 "서브태스크 스택"만 추가.
- **[high]** 이벤트-스트림 아키텍처: 모든 행동/관찰을 하나의 재생가능 append-only 로그로 기록 *(OpenHands)* — "하네싱을 잘해놔야 이상하게 작동을 안한다"에 가장 직접 대응. 다른 기능들이 로그에 기록할 대상(tool 호출)을 먼저 갖춘 뒤 도입.
- **[medium]** Microagents: 자연어 스펙/최소 시연만으로 즉석 생성되는 경량 전용 에이전트 *(OpenHands)* — 오케스트레이터/서브태스크 위임 자리잡은 뒤 확장.
- **[medium]** Custom Modes(페르소나별 시스템프롬프트+도구권한) + "When to Use" 메타데이터 기반 자동 라우팅 *(Roo Code)* — 오케스트레이터 위임이 먼저 있어야 의미 있음.
- **[low]** 한 세션에서 여러 에이전트를 별도 워크트리로 동시 병렬 실행 *(Zed Parallel Agents)* — 오케스트레이션+샌드박싱 안정화 후.

### 6. 검증 & 테스트 기반 루프

- **[high]** 테스트 먼저 작성/도출 → 코드생성 → 테스트실행 → 실패시 수정 tight 루프 *(Micro Agent)* — 기존 edit+명령실행 tool 조합 수준, "자가검증 모드" 플래그 하나로 가능.
- **[medium]** 터미널 명령 실행→에러 파싱→자동 수정 디버그 루프 *(Plandex)* — 위 루프의 일반화.
- **[low]** 여러 후보 패치 생성 후 재현테스트+다수결 선택 *(Agentless)* — LLM 호출 N배, "고신뢰 모드" 옵션으로 나중에.
- **[low]** 스크린샷+로컬URL 시각 비교 반복수정 *(Micro Agent)* — 브라우저 자동화 인프라 별도 필요, 후순위.

### 7. 계획/리뷰 & 승인 UX

- **[high]** Plan(읽기전용)/Act(실행) 모드 명시적 분리 *(Cline, OpenCode)* — tool 실행 진입점 앞 모드 플래그 체크 한 줄, 저비용 고가치 안전장치.
- **[medium]** 실행 전 사람이 읽는 단계별 계획 리스트 표시 *(Devika — UX만 참고)*.
- **[medium]** 플랜을 git 브랜치처럼 저장·재개·비교 가능한 1급 객체로 관리 *(Plandex)* — 세션 영속화 먼저 필요.
- **[low]** 채팅형 계획승인→격리 워크트리 실행→병합/PR/보류 상태기반 UX *(OpenClaw Code Agent)* — 멀티채널 프론트엔드까지 포함하는 큰 그림, core 안정 후.

### 8. 확장성 & 플러그인 아키텍처

- **[high]** 최소 4개 툴(read/write/edit/bash) + 1000토큰 미만 시스템프롬프트 미니멀 코어 *(Pi/badlogic-pi-mono)* — **"넣을 기능"이 아니라 pleumcode 코어 자체의 설계 철학**으로 채택. 다른 모든 기능은 이 얇은 코어 위 선택적 확장이어야 컨텍스트/크레딧 낭비 없음.
- **[high]** 선언적 권한 정책 파일(허용/차단 tool·명령을 모델과 독립 관리) *(Continue CLI)* — YAML/TOML 파싱 수준, 구현비용 매우 낮고 엔터프라이즈 신뢰성에 즉시 기여.
- **[medium]** MCP 서버 원클릭 탐색/설치 마켓플레이스 *(Cline)* — Goose가 이미 MCP 지원하니 UI 계층만.
- **[low]** 코어 로직 전부를 동일 플러그인 인터페이스로 통일(Everything-is-a-plugin) *(DeepSeek Harness)* — core 전체 재설계급, 다른 기능들이 개별 모듈로 존재한 뒤 마지막 통합.
- **[low]** ACP(Agent Client Protocol) 노출로 다른 에디터가 pleumcode를 꽂아 쓰게 *(CodeCompanion.nvim)* — 배포 전략 단계.

### 9. 감사(Auditability) & 운영 가시성

- **[high]** 세션이 작업 디렉토리 기준 자동 저장, 재개/탐색 가능 *(Pi, Continue CLI)* — JSON 하나 쓰는 수준으로 재현성·신뢰 크게 향상.
- **[medium]** 트래젝토리(어떤 플러그인/tool이 뭘 트리거했는지) 전체 실행 이력 감사 뷰 *(DeepSeek Harness)* — 이벤트스트림 아키텍처의 파생 기능.
- **[high]** 세션별 소요시간·비용(크레딧) 노출 운영자 가시성 *(OpenClaw Code Agent)* — PleumRouter가 이미 크레딧 원장 갖고 있어 기존 원장 API 호출 수준.
- **[high]** 선형(linear) 히스토리: 트래젝토리=실제 LLM 전송 메시지, 디버깅 용이 *(mini-SWE-agent)* — 설계 원칙 수준, 비용 거의 없음. 나중에 복잡한 오케스트레이션 추가해도 디버깅 가능성 지키는 전제조건.

### 10. Git & 커밋 자동화

- **[medium]** git 훅(prepare-commit-msg)으로 AI 커밋메시지 무침습 자동생성 *(OpenCommit)* — core와 독립된 서브커맨드로 바로 구현.
- **[high]** 스테이지된 diff만 모델에 전송(미스테이지 변경 비전송) + 리뷰 후 커밋 기본값 *(aicommits)* — 안전 기본값 원칙, pleumcode 모든 자동화 기능(자동커밋/자동PR)에 그대로 적용해야 할 저비용·고가치 안전장치.
- **[low]** git-versioned 마크다운으로 정의한 품질체크를 PR마다 자율에이전트로 실행, GitHub 상태체크로 보고 *(Continue CLI "Continuous AI")* — CI 통합형 부가기능, core 안정 후 조직정책 레이어.

### 11. 온보딩, 배포 & 엔터프라이즈 셀프호스팅

- **[medium]** 설치 없이 즉시 체험 가능한 zero-install 온보딩 UX *(bolt.diy, AgentGPT)* — "PleumRouter 사용자가 한 번 써보게" 목표에 직접 부합. WebContainers 자체는 상용 라이선스 필요해 Rust/로컬 임시 컨테이너로 대체 구현.
- **[low]** 완전 셀프호스팅 단일 바이너리 + 팀 관리자 콘솔(SSO/감사로그) *(Tabby, Refact)* — 엔터프라이즈 배포는 core 완성 후 상품화 단계.
- **[low]** 다중 메신저 채널을 하나의 게이트웨이로 묶어 라우팅 *(OpenClaw)* — 배포 채널 전략에 가까움, 후순위.

### 12. 멀티파일 일관성 & 스캐폴딩 (레거시 패턴)

- **[low]** 파일 생성 전 공유 타입/인터페이스를 마크다운 계약서로 고정 *(smol-developer "shared_dependencies.md")* — repo-map/AST검색이 갖춰지면 상당부분 대체되는 구식 패턴.
- **[low]** clarify→spec→plan→generate→execute 원샷 스캐폴딩 *(GPT Engineer — archived)* — 역사적 UX 참고 자료.

### 13. 라이선스/보안 주의 — 아이디어만, 코드 이식 금지

- AST 구조인지 검색 아이디어 *(AutoCodeRover — SONAR Source-Available, 경쟁사용 금지)* — tree-sitter로 클린룸 재구현.
- 역할기반 멀티에이전트 가상개발팀(PO→Architect→Dev→Reviewer) *(GPT-Pilot — FSL 라이선스+공급망 공격 이력, CodeR — 라이선스 없음)* — 아이디어만, **MetaGPT(MIT)만 코드 참고 가능**.
- 이슈→자동PR 봇 + 소형 오픈웨이트 next-edit 모델 *(Sweep — 현재 Enterprise License)* — UX/모델 배포 아이디어만.

---

## 채택 순서 (구현 시퀀스)

0. **(설계원칙 확정, 코드 아님)** Pi식 "얇은 코어" 철학을 채택하고, GPT-Pilot 공급망 침해 사례 반영해 텔레메트리/원격실행 모듈 서명검증·최소권한 정책을 먼저 문서화.
1. PLEUM.md 계층적 메모리 파일 컨벤션 (core 무변경)
2. 선언적 권한 정책 파일(permissions.yaml류)
3. shadow-git 자동 체크포인트 (파일쓰기 tool wrapper)
4. 편집 적용 전 문법/린트 검사 게이트
5. 스테이지된 diff만 전송 + 리뷰후커밋 안전 기본값을 전 자동화 기능 표준으로
6. 세션 자동 저장/재개(cwd 기준 JSON)
7. Tree-sitter repo-map(PageRank) 모듈
8. AST 기반 구조 인지 코드 검색 tool (클린룸 구현)
9. Pluggable @-context-provider 레지스트리
10. 역할별 모델 설정(plan/editor/apply/autocomplete) + Architect/Editor 분리 + Fast-Apply → Smart Mode/Fusion 라우팅 정책으로 노출
11. Plan/Act 모드 명시적 분리
12. 테스트먼저생성→실행→수정 자가검증 루프(옵션 모드)
13. git 커밋 메시지 자동 생성 서브커맨드
14. 오케스트레이터→컨텍스트 격리 서브태스크 위임(Boomerang) 구조
15. **OS 커널 샌드박스(macOS Seatbelt / Linux Landlock+seccomp)** — approval_policy 분리 (v1 출시 전 필수, [`03-harness-design.md`](./03-harness-design.md) 참고)
16. Agentless류 고정 3단계 저비용 예측가능 모드
17. diff 리뷰 스테이징 샌드박스 + 플랜 브랜치 버전관리 (중기 과제)
18. 이벤트-스트림 아키텍처로 상태관리 재배선
19. Docker 샌드박스 실행 런타임(원격/클라우드 실행 확장)
20. CodeAct식 실행가능 코드 단일 액션 공간 (core급 변경, 후순위)
21. Zed Parallel Agents식 멀티 워크트리 동시 실행
22. Everything-is-a-plugin 최종 통합 리팩터링
23. ACP 등 외부 프로토콜 노출 (배포 전략)
24. zero-install 웹 온보딩 데모 (마케팅 채널, core 완성 후 병행)

---

## 프로젝트 색인 (45개, 참고용)

| 프로젝트 | 라이선스 | 시그니처 기능 (1줄) |
|---|---|---|
| Agentless | MIT | localization→repair→validation 고정 3단계, SWE-bench Lite 27.3% |
| Agentless-Lite | MIT | 임베딩 RAG 상위5파일만, 이슈당 ~$0.21 |
| AgentGPT | GPL-3.0(archived) | 브라우저 네이티브 설치제로 자율에이전트 UX |
| aicommits | MIT | 스테이지 diff만 전송 + 리뷰후커밋 기본값 |
| Aider | Apache-2.0 | repo-map(PageRank) + Architect/Editor 2모델분리 + 모델별 edit-format |
| AutoCodeRover | ⚠️ SONAR Source-Available(비-OSS) | AST 구조인지 검색 + fault localization — 코드 이식 금지 |
| AutoGPT | 혼합(MIT/PolyForm Shield) | thought→plan→critique 자기서술 루프 원조 |
| Avante.nvim | Apache-2.0 | Fast Apply(플래너+고속apply 2모델분리) |
| bolt.diy | MIT(WebContainers는 별도 상용) | 브라우저 내 풀스택 실행, 19+ provider 전환 |
| Cline | Apache-2.0 | Plan/Act 분리 + shadow-git 체크포인트 + MCP 마켓플레이스 |
| CodeAct | MIT | 도구호출을 실행가능 코드 1개로 통일 |
| CodeCompanion.nvim | Apache-2.0 | ACP로 외부 CLI 에이전트를 Neovim에 구동 |
| CodeR | ⚠️ 라이선스 없음(확인불가) | 5역할 그래프 오케스트레이션 |
| Continue | Apache-2.0(archived) | @컨텍스트프로바이더 + config.yaml + CLI/IDE 3-in-1 + Continuous AI 체크 |
| Devika | MIT | plan-then-execute + 웹리서치 서브루프 (유지보수 끊김) |
| Google Gemini CLI | Apache-2.0 | GEMINI.md 계층 메모리(AGENTS.md 원조) + 1M 토큰 컨텍스트 |
| GPT Engineer | MIT(archived) | 프롬프트 1개→전체 스캐폴딩 원조 |
| GPT-Pilot | ⚠️ FSL-1.1-MIT(비-OSS) | 가상개발팀 파이프라인 — 공급망 공격 이력, 반면교사 |
| Lingma SWE-GPT | ⚠️ 라이선스 미확인 | 이슈해결 특화 파인튜닝 SWE 전용 LLM |
| MASAI | ⚠️ 공개 저장소 없음(논문만) | 서브에이전트별 다른 추론전략 배정 아이디어 |
| MetaGPT | MIT | "Code=SOP(Team)" 문서파이프라인 + publish-subscribe 멀티에이전트 |
| Micro Agent | MIT | 테스트먼저생성→구현→검증 tight 루프 |
| Moatless Tools | MIT | AST 정밀 위치탐색 + 이슈당 $0.01~0.14 비용효율 |
| OpenAI Codex CLI | Apache-2.0 | OS커널 샌드박스(Seatbelt/Landlock+seccomp) + AGENTS.md 대중화 |
| OpenClaw | MIT | 20+ 메신저채널 통합 개인비서 게이트웨이 |
| OpenClaw Code Agent | MIT | 채팅형 계획승인→워크트리→PR 상태기반 UX |
| OpenCode (sst) | MIT | build/plan Tab전환 + general subagent |
| OpenCommit | MIT | git 훅 무침습 커밋메시지 자동화 |
| OpenHands | MIT | 이벤트스트림 + CodeActAgent + Docker 샌드박스 + Microagents |
| Plandex | MIT | diff리뷰 샌드박스 + 플랜 브랜치버전관리 + 20M토큰 레포인덱싱 |
| Qwen Code | Apache-2.0 | 4-프로토콜 네이티브 + Agent Teams/SubAgents + Auto-Memory |
| Refact.ai | BSD-3-Clause | Rust 로컬 에이전트런타임(AST+VecDB) + 셀프호스팅 |
| RepoGraph | Apache-2.0 | 저장소 호출/임포트 그래프를 플러그인처럼 얹는 모듈 |
| Roo Code | Apache-2.0 | Boomerang Tasks(컨텍스트격리 서브태스크 위임) + Custom Modes |
| smol-developer | MIT | shared_dependencies.md로 멀티파일 일관성 확보 |
| Sourcegraph Cody | Apache-2.0(dormant) | OpenCtx 프로토콜 + 코드그래프 기반 컨텍스트 |
| SWE-agent | MIT | ACI(LM전용 편집커맨드) + 린트게이트, sibling mini-SWE-agent |
| Sweep | ⚠️ 현재 대부분 Enterprise License | 이슈→자동PR — 코드 이식 금지 |
| Tabby | Apache-2.0 | 완전 셀프호스팅 단일 Rust바이너리 + 로컬RAG |
| Void | Apache-2.0(archived) | 편집단위 체크포인트 롤백 + 프록시없는 BYOK |
| Zed | GPL-3.0/Apache-2.0 | Parallel Agents + Zeta 자체호스팅 edit-prediction |
| mini-SWE-agent | MIT | 순수파이썬 ~100줄, bash 툴 1개, SWE-bench Verified 74%+ |
| DeepSeek Harness | MIT | Everything-is-a-plugin(Cordis), creator 모드 |
| Pi (badlogic/pi-mono) | MIT | 4개 툴만 + 1000토큰 미만 프롬프트 극단 미니멀리즘 |
